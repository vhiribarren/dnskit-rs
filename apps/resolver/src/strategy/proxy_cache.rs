/*
MIT License

Copyright (c) 2026 Vincent Hiribarren

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use anyhow::Context;
use async_trait::async_trait;
use dnskit::protocol::{
    allocate_udp_recv_buffer,
    message::{Header, Message, OpCode, QueryResponse, Question, ResourceRecord, ResponseCode},
    parser::parse,
};
use hex::ToHex;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::UdpSocket;
use tracing::{debug, info, instrument, trace, warn};

use crate::{
    cache::DnsCache,
    strategy::{LOCAL_ADDR, ProcessStrategy},
};

pub struct ProxyCacheStrategy<C> {
    socket_addr: SocketAddr,
    cache: Mutex<C>,
}

impl<C: DnsCache> ProxyCacheStrategy<C> {
    pub fn new(socket_addr: SocketAddr, cache: C) -> Self {
        info!("Proxy strategy configured with target address: {socket_addr}");
        Self {
            socket_addr,
            cache: Mutex::new(cache),
        }
    }

    async fn get_or_resolve(&self, questions: &[Question]) -> anyhow::Result<Vec<ResourceRecord>> {
        let cached_questions = {
            let mut locked_cache = self
                .cache
                .lock()
                .expect("Some thread panicked, stopping program");
            questions
                .iter()
                .map(|q| (locked_cache.get(q), q))
                .collect::<Vec<_>>()
        };

        let mut answers = Vec::new();
        for (cached_answer, question) in cached_questions {
            if let Some(mut answer) = cached_answer {
                debug!(question = ?question, answer = ?answer, "in cache");
                answers.append(&mut answer);
                continue;
            }
            debug!(question = ?question, "not in cache, querying {}", self.socket_addr);
            let proxied_rmessage = self.resolve_and_cache(question).await?;
            check_response_valid(&proxied_rmessage, &self.socket_addr)?;
            answers.extend_from_slice(&proxied_rmessage.answers);
            self.cache
                .lock()
                .expect("Some thread panicked, stopping program")
                .replace(question, proxied_rmessage.answers);
        }
        Ok(answers)
    }

    async fn resolve_and_cache(&self, question: &Question) -> anyhow::Result<Message> {
        let proxied_qmessage = Message {
            header: Header {
                id: 42,
                query_response: QueryResponse::Query,
                opcode: OpCode::Query,
                authoritative_answer: false,
                truncation: false,
                recursion_desired: true,
                recursion_available: false,
                reserved: false,
                authentic_data: false,
                checking_disabled: false,
                response_code: ResponseCode::NoErrorCondition,
            },
            questions: vec![question.clone()],
            answers: Vec::new(),
            authority: Vec::new(),
            additional: Vec::new(),
        };
        let serialized_proxied_qmessage = &proxied_qmessage.serialize();
        trace!(
            from = %LOCAL_ADDR,
            to = %self.socket_addr,
            len = serialized_proxied_qmessage.len(),
            payload = serialized_proxied_qmessage.encode_hex_upper::<String>(),
            message = ?proxied_qmessage,
            "query sent to resolver"
        );

        let client_socket = UdpSocket::bind(LOCAL_ADDR).await?;
        client_socket.connect(self.socket_addr).await?;

        let mut recv_buffer = allocate_udp_recv_buffer();
        client_socket.send(serialized_proxied_qmessage).await?;
        let recv_len = client_socket.recv(&mut recv_buffer).await?;
        let recv_slice = &recv_buffer[..recv_len];
        let proxied_rmessage = parse(recv_slice)?;
        trace!(
            from = %self.socket_addr,
            to = %LOCAL_ADDR,
            len = recv_len,
            payload = recv_slice.encode_hex_upper::<String>(),
            message = ?proxied_rmessage,
            "response received from resolver"
        );
        Ok(proxied_rmessage)
    }
}

#[async_trait]
impl<C: DnsCache + Send + Sync> ProcessStrategy for ProxyCacheStrategy<C> {
    #[instrument(skip_all)]
    async fn process_recv_data(
        &self,
        buffer: Vec<u8>,
        src_addr: SocketAddr,
        socket: Arc<UdpSocket>,
    ) -> anyhow::Result<()> {
        let qmessage = parse(&buffer)?;
        let questions = &qmessage.questions;
        info!(from = %src_addr, ?questions, "query received");
        trace!(
            from = %src_addr,
            len = buffer.len(),
            payload = buffer.encode_hex_upper::<String>(),
            message = ?qmessage
        );

        check_query_valid(&qmessage, &src_addr)?;

        let answers = self.get_or_resolve(questions).await?;

        let rmessage = Message {
            header: Header {
                id: qmessage.header.id,
                query_response: QueryResponse::Response,
                opcode: OpCode::Query,
                authoritative_answer: false,
                truncation: false,
                recursion_desired: false,
                recursion_available: true,
                reserved: false,
                authentic_data: false,
                checking_disabled: false,
                response_code: ResponseCode::NoErrorCondition,
            },
            questions: questions.clone(),
            answers: answers.clone(),
            authority: Vec::new(),
            additional: Vec::new(),
        };
        let serialized_rmessage = rmessage.serialize();

        info!(to = %src_addr, ?questions, ?answers, "response sent");
        trace!(
            to = %src_addr,
            len = serialized_rmessage.len(),
            payload = serialized_rmessage.encode_hex_upper::<String>(),
            message = ?rmessage
        );
        socket.send_to(&serialized_rmessage, src_addr).await?;
        Ok(())
    }
}

fn check_query_valid(qmessage: &Message, src_addr: &SocketAddr) -> anyhow::Result<()> {
    let question = qmessage.questions.first().context("No questions header")?;
    if qmessage.header.query_response != QueryResponse::Query {
        warn!(
                from = %src_addr,
                qclass = ?question.qclass,
                qtype = ?question.qtype,
                qname = question.name(),
                "was waiting for a query, but has response flag");
    }
    if qmessage.questions.len() != 1 {
        warn!(
            count = qmessage.questions.len(),
            "query do not have 1 query entry"
        );
    }
    Ok(())
}

fn check_response_valid(rmessage: &Message, socket_addr: &SocketAddr) -> anyhow::Result<()> {
    if rmessage.header.query_response != QueryResponse::Response {
        warn!(
                from = %socket_addr,
                "was waiting for a response, but has query flag");
    }
    if rmessage.questions.len() != 1 {
        warn!(
            count = rmessage.questions.len(),
            "answer do not have 1 query entry"
        );
    }
    Ok(())
}

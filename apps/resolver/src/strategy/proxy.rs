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

use dnskit::protocol::{
    allocate_udp_recv_buffer,
    message::{Message, QueryResponse},
    parser::parse,
};
use hex::ToHex;
use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::net::UdpSocket;
use tracing::{debug, info, instrument, trace, warn};

use crate::strategy::ProcessStrategy;

const LOCAL_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
const TARGET_PROXY_ADDR_DEFAULT: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53);

#[derive(Clone)]
pub struct ProxyStrategy {
    socket_addr: SocketAddr,
}

impl ProxyStrategy {
    pub fn new(socket_addr: SocketAddr) -> Self {
        info!("Proxy strategy configured with target address: {socket_addr}");
        ProxyStrategy { socket_addr }
    }
}

impl Default for ProxyStrategy {
    fn default() -> Self {
        Self::new(TARGET_PROXY_ADDR_DEFAULT)
    }
}

impl ProcessStrategy for ProxyStrategy {
    #[instrument(skip_all)]
    async fn process_recv_data(
        &self,
        buffer: Vec<u8>,
        src_addr: SocketAddr,
        socket: Arc<UdpSocket>,
    ) -> io::Result<()> {
        let qmessage = parse(&buffer).unwrap();
        check_query_valid(&qmessage, &src_addr)?;
        trace!(
            len = buffer.len(),
            payload = buffer.encode_hex_upper::<String>()
        );
        debug!(?qmessage);

        let client_socket = UdpSocket::bind(LOCAL_ADDR).await?;
        client_socket.connect(self.socket_addr).await?;

        let mut recv_buffer = allocate_udp_recv_buffer();
        client_socket.send(&buffer).await?;
        let recv_len = client_socket.recv(&mut recv_buffer).await?;

        let rmessage = parse(&recv_buffer[..recv_len]).unwrap();
        check_response_valid(&rmessage, &self.socket_addr)?;
        trace!(to = %src_addr, len = recv_len, payload = (&recv_buffer[..recv_len]).encode_hex_upper::<String>(), "response");
        debug!(message = ?parse(&recv_buffer), "response");

        socket.send_to(&recv_buffer[..recv_len], src_addr).await?;
        Ok(())
    }
    
}

fn check_query_valid(qmessage: &Message, src_addr: &SocketAddr) -> io::Result<()> {
    let question = qmessage.questions.get(0).unwrap();
    if qmessage.header.query_response == QueryResponse::Query {
        info!(
                from = %src_addr,
                qclass = ?question.qclass,
                qtype = ?question.qtype,
                qname = question.name(),
                "query received");
    } else {
        warn!(
                from = %src_addr,
                qclass = ?question.qclass,
                qtype = ?question.qtype,
                qname = question.name(),
                "was waiting for a query, but has response flag");
    }
    if qmessage.questions.len() != 1 {
        warn!(count = qmessage.questions.len() , "query do not have 1 query entry");
    }
    Ok(())
}

fn check_response_valid(rmessage: &Message, socket_addr: &SocketAddr) -> io::Result<()> {
    if rmessage.header.query_response == QueryResponse::Response {
        info!(
                from = %socket_addr,
                "response received");
    } else {
        warn!(
                from = %socket_addr,
                "was waiting for a response, but has query flag");
    }
    if rmessage.questions.len() != 1 {
        warn!(count = rmessage.questions.len() , "answer do not have 1 query entry");
    }
    Ok(())
}

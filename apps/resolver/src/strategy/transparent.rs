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

use async_trait::async_trait;
use dnskit::protocol::{allocate_udp_recv_buffer, parser::parse};
use hex::ToHex;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;
use tracing::{info, instrument, trace, warn};

use crate::strategy::{LOCAL_ADDR, ProcessStrategy};

pub struct TransparentProxyStrategy {
    socket_addr: SocketAddr,
}

impl TransparentProxyStrategy {
    pub fn new(socket_addr: SocketAddr) -> Self {
        info!("Transparent Proxy strategy configured with target address: {socket_addr}");
        TransparentProxyStrategy { socket_addr }
    }
}

#[async_trait]
impl ProcessStrategy for TransparentProxyStrategy {
    #[instrument(skip_all)]
    async fn process_recv_data(
        &self,
        buffer: Vec<u8>,
        src_addr: SocketAddr,
        socket: Arc<UdpSocket>,
    ) -> anyhow::Result<()> {
        let qmessage = parse(&buffer)?;
        let rquestions = &qmessage.questions;
        info!(from = %src_addr, ?rquestions, "query received");
        trace!(
            from = %src_addr,
            len = buffer.len(),
            payload = buffer.encode_hex_upper::<String>(),
            message = ?qmessage
        );

        // TODO Option to enable/disable check since transparent mode?
        if let Err(err) = qmessage.check_valid_query() {
            warn!(?err, ?qmessage, "invalid message, but continue processing");
        }

        let client_socket = UdpSocket::bind(LOCAL_ADDR).await?;
        client_socket.connect(self.socket_addr).await?;

        let mut recv_buffer = allocate_udp_recv_buffer();
        client_socket.send(&buffer).await?;
        let recv_len = client_socket.recv(&mut recv_buffer).await?;
        let recv_slice = &recv_buffer[..recv_len];

        let rmessage = parse(recv_slice)?;
        let rquestions = &rmessage.questions;
        let answers = &rmessage.answers;
        // TODO Option to enable/disable check since transparent mode?
        if let Err(err) = rmessage.check_valid_response() {
            warn!(?err, ?rmessage, "invalid message, but continue processing");
        }
        info!(to = %src_addr, ?rquestions, ?answers, "response sent");
        trace!(
            to = %src_addr,
            len = recv_len,
            payload = recv_slice.encode_hex_upper::<String>(),
            message = ?rmessage
        );

        socket.send_to(recv_slice, src_addr).await?;
        Ok(())
    }
}

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

use dnskit::protocol::allocate_udp_recv_buffer;
use hex::ToHex;
use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::net::UdpSocket;
use tracing::{info, trace};

use crate::strategy::ProcessStrategy;

const TARGET_PROXY_ADDR_DEFAULT: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53);
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
    async fn process_recv_data(
        &self,
        buffer: Vec<u8>,
        src_addr: SocketAddr,
        socket: Arc<UdpSocket>,
    ) -> io::Result<()> {
        info!(from = %src_addr, "request received");
        trace!(from = %src_addr, len = buffer.len(), payload = buffer.encode_hex_upper::<String>());

        let client_socket = UdpSocket::bind("0.0.0.0:0").await?;
        client_socket.connect(self.socket_addr).await?;

        let mut recv_buffer = allocate_udp_recv_buffer();
        client_socket.send(&buffer).await?;
        let recv_len = client_socket.recv(&mut recv_buffer).await?;

        trace!(to = %src_addr, len = recv_len, payload = (&recv_buffer[..recv_len]).encode_hex_upper::<String>(), "response");
        socket.send_to(&recv_buffer[..recv_len], src_addr).await?;

        Ok(())
    }
}

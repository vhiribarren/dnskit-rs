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
        trace!(from = %src_addr, len = buffer.len(), payload = buffer.encode_hex_upper::<String>(), "request");

        let client_socket = UdpSocket::bind("0.0.0.0:0").await?;
        client_socket.connect(self.socket_addr).await?;

        let mut recv_buffer = allocate_udp_recv_buffer();
        client_socket.send(&buffer).await?;
        let recv_len = client_socket.recv(&mut recv_buffer).await?;

        trace!(to = %src_addr, len = recv_len, payload = (&recv_buffer[..recv_len]).encode_hex_upper::<String>(), "response");
        socket.send_to(&recv_buffer, src_addr).await?;

        Ok(())
    }
}

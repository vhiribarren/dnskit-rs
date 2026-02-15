use hex::ToHex;
use std::{io, net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;
use tracing::{debug, info, trace};

use crate::strategy::ProcessStrategy;

type RecvBuffer = [u8; RECV_BUFFER_SIZE];

const TARGET_ADDR_DEFAULT: &str = "8.8.8.8:53";
const RECV_BUFFER_SIZE: usize = 512;

pub struct ProxyStrategy;

impl ProcessStrategy for ProxyStrategy {
    async fn process_recv_data(
        &self,
        buffer: Vec<u8>,
        src_addr: SocketAddr,
        socket: Arc<UdpSocket>,
    ) -> io::Result<()> {
        trace!(from = %src_addr, len = buffer.len(), payload = buffer.encode_hex_upper::<String>(), "request");

        let client_socket = UdpSocket::bind("0.0.0.0:0").await?;
        client_socket.connect(TARGET_ADDR_DEFAULT).await?;

        let mut recv_buffer: RecvBuffer = [0; RECV_BUFFER_SIZE];
        client_socket.send(&buffer).await?;
        let recv_len = client_socket.recv(&mut recv_buffer).await?;

        trace!(to = %src_addr, len = recv_len, payload = (&recv_buffer[..recv_len]).encode_hex_upper::<String>(), "response");
        socket.send_to(&recv_buffer, src_addr).await?;

        Ok(())
    }
}

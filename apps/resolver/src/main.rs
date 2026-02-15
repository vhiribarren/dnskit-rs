mod strategy;

use std::{io, sync::Arc};
use tokio::net::{ToSocketAddrs, UdpSocket};
use tracing::{debug, info, level_filters::LevelFilter, trace};
use tracing_subscriber::EnvFilter;

use crate::strategy::ProcessStrategy;
use crate::strategy::proxy::ProxyStrategy;

const SOCKET_ADDR_DEFAULT: &str = "127.0.0.1:3553";
const RECV_BUFFER_SIZE: usize = 512;

type RecvBuffer = [u8; RECV_BUFFER_SIZE];

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    info!("Starting DNS server on {SOCKET_ADDR_DEFAULT}");
    launch_server(SOCKET_ADDR_DEFAULT).await
}

async fn launch_server<A>(local_addr: A) -> io::Result<()>
where
    A: ToSocketAddrs,
{
    let process_strategy = Arc::new(ProxyStrategy);
    let socket = Arc::new(UdpSocket::bind(local_addr).await?);
    loop {
        let mut recv_buffer: RecvBuffer = [0; RECV_BUFFER_SIZE];
        let (recv_len, recv_addr) = socket.recv_from(&mut recv_buffer).await?;
        info!(len = recv_len, from = ?recv_addr, "request received");

        let local_processor = Arc::clone(&process_strategy);
        let local_socket = Arc::clone(&socket);
        tokio::spawn(async move {
            local_processor
                .process_recv_data(recv_buffer[..recv_len].to_vec(), recv_addr, local_socket)
                .await
        });
    }
}

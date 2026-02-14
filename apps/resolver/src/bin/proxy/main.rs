use hex::ToHex;
use std::{io, net::SocketAddr, sync::Arc};
use tokio::net::{ToSocketAddrs, UdpSocket};
use tracing::{debug, info, level_filters::LevelFilter, trace};
use tracing_subscriber::EnvFilter;

const SOCKET_ADDR_DEFAULT: &str = "127.0.0.1:3553";
const TARGET_ADDR_DEFAULT: &str = "8.8.8.8:53";
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
    info!("Starting Proxy DNS");
    launch_proxy(SOCKET_ADDR_DEFAULT).await
}

async fn launch_proxy<A>(local_addr: A) -> io::Result<()>
where
    A: ToSocketAddrs,
{
    let socket = Arc::new(UdpSocket::bind(local_addr).await?);
    loop {
        let mut recv_buffer: RecvBuffer = [0; RECV_BUFFER_SIZE];
        let (recv_len, recv_addr) = socket.recv_from(&mut recv_buffer).await?;
        debug!(len = recv_len, from = ?recv_addr, "request received");
        tokio::spawn(process_recv_data(
            recv_buffer[..recv_len].to_vec(),
            recv_addr,
            Arc::clone(&socket),
        ));
    }
}

async fn process_recv_data(
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

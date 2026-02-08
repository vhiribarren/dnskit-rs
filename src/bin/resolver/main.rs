use dnskit_rs::protocol::parser::parse;
use std::{io, net::SocketAddr, sync::Arc};
use tokio::net::{ToSocketAddrs, UdpSocket};
use tracing::{debug, info, level_filters::LevelFilter, trace};
use tracing_subscriber::EnvFilter;

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
    info!("Starting DNS resolver");
    launch_server(SOCKET_ADDR_DEFAULT).await
}

async fn launch_server<A>(local_addr: A) -> io::Result<()>
where
    A: ToSocketAddrs,
{
    let socket = Arc::new(UdpSocket::bind(local_addr).await?);
    loop {
        let mut recv_buffer: RecvBuffer = [0; RECV_BUFFER_SIZE];
        let (recv_len, recv_addr) = socket.recv_from(&mut recv_buffer).await?;
        debug!(recv_len, ?recv_addr, "bytes received");
        tokio::spawn(process_recv_data(
            recv_len,
            recv_buffer,
            recv_addr,
            Arc::clone(&socket),
        ));
    }
}

async fn process_recv_data(
    len: usize,
    buffer: RecvBuffer,
    src_addr: SocketAddr,
    socket: Arc<UdpSocket>,
) -> io::Result<()> {
    let message = parse(&buffer).unwrap();
    trace!(?message);
    let len = socket.send_to(&buffer[..len], src_addr).await?;
    debug!(len, "bytes sent");
    Ok(())
}

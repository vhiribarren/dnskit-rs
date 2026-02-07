use std::{io, net::SocketAddr, sync::Arc};
use tokio::net::{ToSocketAddrs, UdpSocket};

const SOCKET_ADDR_DEFAULT: &str = "127.0.0.1:3553";
const RECV_BUFFER_SIZE: usize = 512;

type RecvBuffer = [u8; RECV_BUFFER_SIZE];

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
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
        println!("{:?} bytes received from {:?}", recv_len, recv_addr);
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
    let len = socket.send_to(&buffer[..len], src_addr).await?;
    println!("{:?} bytes sent", len);
    Ok(())
}

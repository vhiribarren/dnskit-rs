pub mod proxy;

use std::{net::SocketAddr, sync::Arc};
use tokio::{io, net::UdpSocket};

pub trait ProcessStrategy {
    async fn process_recv_data(
        &self,
        buffer: Vec<u8>,
        src_addr: SocketAddr,
        socket: Arc<UdpSocket>,
    ) -> io::Result<()>;
}

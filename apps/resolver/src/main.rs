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

mod strategy;

use dnskit::protocol::allocate_udp_recv_buffer;
use std::{io, sync::Arc};
use tokio::net::{ToSocketAddrs, UdpSocket};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

use crate::strategy::ProcessStrategy;
use crate::strategy::proxy::ProxyStrategy;

const SOCKET_ADDR_DEFAULT: &str = "127.0.0.1:3553";

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
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
    let process_strategy = Arc::new(ProxyStrategy::default());
    let socket = Arc::new(UdpSocket::bind(local_addr).await?);
    loop {
        let mut recv_buffer = allocate_udp_recv_buffer();
        let (recv_len, recv_addr) = socket.recv_from(&mut recv_buffer).await?;

        let local_processor = Arc::clone(&process_strategy);
        let local_socket = Arc::clone(&socket);
        tokio::spawn(async move {
            local_processor
                .process_recv_data(recv_buffer[..recv_len].to_vec(), recv_addr, local_socket)
                .await
        });
    }
}

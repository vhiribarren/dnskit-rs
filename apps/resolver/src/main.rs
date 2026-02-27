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

mod cache;
mod strategy;

use clap::Parser;
use dnskit::protocol::allocate_udp_recv_buffer;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::{UdpSocket, lookup_host};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

use crate::cache::memory::DnsCacheMemory;
use crate::strategy::ProcessStrategy;
use crate::strategy::proxy::ProxyStrategy;
use crate::strategy::proxy_cache::ProxyCacheStrategy;

const SOCKET_ADDR_DEFAULT: &str = "127.0.0.1:3553";
const TARGET_PROXY_ADDR_DEFAULT: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53);
    
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    #[arg(long)]
    no_cache: bool,

    #[arg(long)]
    proxy: bool,

    /// (only valid with --proxy)
    #[arg(long, requires = "proxy")]
    passthrough: bool,

    /// Target host (only valid with --proxy)
    #[arg(long, requires = "proxy")]
    target_host: Option<String>,

    /// Target port (only valid with --proxy)
    #[arg(long, requires = "proxy")]
    target_port: Option<u16>,
}



#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    println!("\nStarting DnsKit Resolver - use -v for more verbose output.\n");
    setup_log(args.verbose);

    let process_strategy: Arc<dyn ProcessStrategy> = {
        if args.proxy {
            let target_sockaddr =  if let Some(target_host) = args.target_host {
                lookup_host((target_host, args.target_port.unwrap_or(53))).await?.next().unwrap()
            }
            else {
                TARGET_PROXY_ADDR_DEFAULT
            };
            if args.passthrough {
                Arc::new(ProxyStrategy::new(target_sockaddr))
            }
            else {
                Arc::new(ProxyCacheStrategy::new(target_sockaddr, DnsCacheMemory::new()))
            }
        }
        else {
            unimplemented!()
        }
    };

    launch_server(process_strategy).await
}


fn setup_log(verbose_count: u8) {
    let log_level = match verbose_count {
        0 => LevelFilter::WARN,
        1 => LevelFilter::INFO,
        2 => LevelFilter::DEBUG,
        _ => LevelFilter::TRACE,
    };
    let env_filter = EnvFilter::builder()
        .with_default_directive(log_level.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(env_filter)
        .init();
}


async fn launch_server(process_strategy: Arc<dyn ProcessStrategy>) -> anyhow::Result<()>
{
    info!("Starting DNS server on {SOCKET_ADDR_DEFAULT}");
    let socket = Arc::new(UdpSocket::bind(SOCKET_ADDR_DEFAULT).await?);
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

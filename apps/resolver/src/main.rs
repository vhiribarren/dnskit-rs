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
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{UdpSocket, lookup_host};
use tracing::{debug, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

use crate::cache::memory::DnsCacheMemory;
use crate::strategy::ProcessStrategy;
use crate::strategy::proxy_cache::ProxyCacheStrategy;
use crate::strategy::transparent::TransparentProxyStrategy;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const DNS_PORT: u16 = 53;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, next_line_help = true, disable_help_subcommand = true)]
struct CliArgs {
    /// More execution information, up to -vvv
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Local IP address to use.
    #[arg(long, default_value = "127.0.0.1")]
    local_ip: String,

    /// Local port to use.
    #[arg(long, default_value_t = 3553)]
    local_port: u16,

    /// Disable local cache, resolve all requests.
    #[arg(long)]
    no_cache: bool,

    /// Forward requests to another resolver selected with --target-host.
    #[arg(long)]
    proxy: bool,

    #[command(flatten, next_help_heading = "Proxy Options")]
    proxy_args: ProxyArgs,
}

#[derive(clap::Args, Debug)]
#[group(requires = "proxy")]
struct ProxyArgs {
    /// Requests are transmitted as is to another resolver,
    /// without modifications. Local cache is disabled.
    #[arg(long)]
    transparent: bool,

    /// Target host when proxy mode is enabled.
    #[arg(long, default_value = "8.8.8.8")]
    target_host: String,

    /// Target port when proxy mode is enabled.
    #[arg(long,  default_value_t = DNS_PORT)]
    target_port: u16,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();

    println!("\n");
    println!("Starting {APP_NAME} {APP_VERSION}.");
    println!("Options --help for more parameters, -v for more outputs.");
    println!("\n");

    let local_sockaddr = SocketAddr::new(args.local_ip.parse()?, args.local_port);

    println!("Listening on {local_sockaddr}");
    println!("\n");

    let local_socket = Arc::new(UdpSocket::bind(local_sockaddr).await?);
    setup_log(args.verbose);
    debug!(?args, "parameters");

    let process_strategy: Arc<dyn ProcessStrategy> = {
        if args.proxy {
            let target_sockaddr =
                lookup_host((args.proxy_args.target_host, args.proxy_args.target_port))
                    .await?
                    .next()
                    .unwrap();
            if args.proxy_args.transparent {
                Arc::new(TransparentProxyStrategy::new(target_sockaddr))
            } else {
                Arc::new(ProxyCacheStrategy::new(
                    target_sockaddr,
                    DnsCacheMemory::new(),
                ))
            }
        } else {
            unimplemented!()
        }
    };

    launch_server(local_socket, process_strategy).await
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

async fn launch_server(
    local_socket: Arc<UdpSocket>,
    process_strategy: Arc<dyn ProcessStrategy>,
) -> anyhow::Result<()> {
    loop {
        let mut recv_buffer = allocate_udp_recv_buffer();
        let (recv_len, recv_addr) = local_socket.recv_from(&mut recv_buffer).await?;

        let local_task_processor = Arc::clone(&process_strategy);
        let local_task_socket = Arc::clone(&local_socket);
        tokio::spawn(async move {
            local_task_processor
                .process_recv_data(
                    recv_buffer[..recv_len].to_vec(),
                    recv_addr,
                    local_task_socket,
                )
                .await
        });
    }
}

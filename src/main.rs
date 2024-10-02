mod packet;
mod header;
mod result_code;
mod question;
mod query_type;
mod record;
mod parser;
mod writer;
mod bytes_util;
mod pair;
mod query_class;
mod server;
mod resolver;
mod root;
mod context;
mod handler;
mod cache;

use clap::{Parser};
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;
use crate::server::{DnsServer, UdpDnsServer};

#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    #[arg(long, short, default_value_t = 53)]
    port: u16,
    #[arg(long, short, value_parser, num_args = 1.., value_delimiter = ',')]
    forward: Option<Vec<String>>
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder().with_max_level(Level::TRACE).finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize logger");

    let args = Args::parse();

    let dns_server = UdpDnsServer::new(args.port, args.forward);
    if let Err(e) = dns_server.start() {
        error!("Failed to start dns server: {}", e.to_string())
    }
}

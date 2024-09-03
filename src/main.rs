mod packet;
mod header;
mod result_code;
mod question;
mod dns_type;
mod record;
mod parser;
mod writer;
mod bytes_util;
mod pair;
mod dns_class;
mod server;
mod resolver;

use std::net::{IpAddr, SocketAddr, UdpSocket};
use crate::parser::PacketParser;
use clap::{Parser};
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use crate::packet::Packet;
use crate::server::DnsServer;
use crate::writer::PacketWriter;

#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    #[arg(long, short, default_value_t = 53)]
    port: u16,
    #[arg(long, short, default_value = "8.8.8.8")]
    forward: String
}

fn main() {
    let subscriber = FmtSubscriber::builder().with_max_level(Level::TRACE).finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize logger");

    let args = Args::parse();

    let dns_server = DnsServer::new(args.port, &args.forward);
    if let Err(e) = dns_server.start() {
        error!("Failed to start dns server: {}", e.to_string())
    }
}

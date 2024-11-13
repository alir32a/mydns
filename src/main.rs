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
mod zone;
mod args;
mod duration;

use clap::{Parser};
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;
use crate::args::Args;
use crate::context::Context;
use crate::server::{DnsServer, UdpDnsServer};

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder().with_max_level(Level::TRACE).finish();
    
    tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize logger");

    let args = Args::parse();
    
    let ctx = Context::from(args).unwrap();

    println!(r"
   __  _____  _____  _  ______
  /  |/  /\ \/ / _ \/ |/ / __/
 / /|_/ /  \  / // /    /\ \  
/_/  /_/   /_/____/_/|_/___/
    ");

    let dns_server = UdpDnsServer::new(ctx);
    if let Err(e) = dns_server.start().await {
        error!("Failed to start dns server: {}", e.to_string())
    }
}

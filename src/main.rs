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
mod record_data;
mod dns_class;

use std::net::{IpAddr, SocketAddr, UdpSocket};
use crate::parser::PacketParser;
use clap::{Parser};
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use crate::packet::Packet;
use crate::writer::PacketWriter;

#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    #[arg(long, short, default_value_t = 53)]
    port: u16,
}

fn main() {
    let subscriber = FmtSubscriber::builder().with_max_level(Level::TRACE).finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize logger");

    let args = Args::parse();

    let udp_socket = match UdpSocket::bind(SocketAddr::new(IpAddr::from([0,0,0,0]), args.port)) {
        Ok(socket) => socket,
        Err(err) => {
            error!("Failed to start server: {}", err);

            std::process::exit(1);
        }
    };
    info!("Listening on 0.0.0.0:{}", args.port);
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((_size, source)) => {
                let mut parser = PacketParser::new(&buf);
                let packet = match parser.parse() {
                    Ok(packet) => packet,
                    Err(err) => {
                        error!("Failed to parse a packet: {}", err);

                        continue;
                    },
                };

                let res_packet = Packet::from(packet);

                udp_socket
                    .send_to(PacketWriter::from(res_packet).write().unwrap(), source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}

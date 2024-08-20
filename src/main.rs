mod packet;
mod header;
mod result_code;
mod util;
mod question;
mod query_type;
mod record;
mod packet_parser;

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use crate::packet::Packet;
use crate::packet_parser::PacketParser;
use clap::{Parser};
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

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
            Ok((size, source)) => {
                let mut parser = PacketParser::new(&buf);
                let packet = match parser.parse() {
                    Ok(packet) => packet,
                    Err(err) => {
                        error!("Failed to parse a packet: {}", err);

                        continue;
                    },
                };

                info!("got a packet");
                info!("header: {:?}", packet.header);

                for rec in packet.answers {
                    info!("answer record: {:?}", rec);
                }

                for rec in packet.authorities {
                    info!("authorities record: {:?}", rec);
                }

                for rec in packet.resources {
                    info!("resources record: {:?}", rec);
                }

                udp_socket
                    .send_to(&parser.bytes(), source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}

use std::net::{IpAddr, SocketAddr, UdpSocket};
use anyhow::{bail, Result};
use tracing::{error, info};
use crate::packet::Packet;
use crate::parser::PacketParser;
use crate::stub_resolver::StubResolver;
use crate::writer::PacketWriter;

pub struct DnsServer<'s> {
    pub port: u16,
    forward: &'s str,
}

impl<'s> DnsServer<'s> {
    pub fn new(port: u16, forward: &'s str) -> DnsServer {
        Self {
            port,
            forward
        }
    }

    pub fn start(&self) -> Result<()> {
        let udp_socket = match UdpSocket::bind(SocketAddr::new(IpAddr::from([0,0,0,0]), self.port)) {
            Ok(socket) => socket,
            Err(err) => {
                error!("Failed to start server: {}", err);

                std::process::exit(1);
            }
        };
        info!("Listening on 0.0.0.0:{}", self.port);
        let mut buf = [0; 512];
        let mut resolver: Option<StubResolver> = None;

        if !self.forward.is_empty() {
            resolver = Some(StubResolver::new(self.forward));
        }

        loop {
            match udp_socket.recv_from(&mut buf) {
                Ok((_size, source)) => {
                    if let Some(resolver) = &resolver {
                        let res = match resolver.resolve(&buf) {
                            Ok(res) => res,
                            Err(e) => {
                                error!("Resolve error: {}", e.to_string());

                                continue;
                            }
                        };

                        info!("Forwarded a request to {}", resolver.forward);

                        udp_socket.send_to(res.as_slice(), source).expect("Failed to send response");
                        continue;
                    }

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
                    error!("Error receiving data: {}", e);
                }
            }
        }
    }
}
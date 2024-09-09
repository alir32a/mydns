use std::net::{IpAddr, UdpSocket};
use std::str::FromStr;
use anyhow::{Result};
use tracing::{error, info};
use crate::handler::{Handler, HandlerTarget, UdpHandler};
use crate::packet::Packet;
use crate::parser::PacketParser;
use crate::resolver::Resolver;
use crate::root::{get_root_servers_handle_targets, ROOT_SERVERS};
use crate::writer::PacketWriter;

pub trait DnsServer {
    fn start(&self) -> Result<()>;
}

pub struct UdpDnsServer {
    pub port: u16,
    forward: Option<Vec<String>>,
}

impl UdpDnsServer {
    pub fn new(port: u16, forward: Option<Vec<String>>) -> UdpDnsServer {
        Self {
            port,
            forward
        }
    }

    fn log_response_packet(res: &[u8]) -> Result<()> {
        let packet = PacketParser::new(res).parse()?;
        info!("Parsed packet header: {:?}", packet.header);

        for question in packet.questions {
            info!("Parsed packet question: {:?}", question);
        }

        for answer in packet.answers {
            info!("Parsed packet answer: {:?}", answer);
        }

        for authority in packet.authorities {
            info!("Parsed packet authority: {:?}", authority);
        }

        for resource in packet.resources {
            info!("Parsed packet resource: {:?}", resource);
        }

        Ok(())
    }
}

impl DnsServer for UdpDnsServer {
    fn start(&self) -> Result<()> {
        let udp_socket = match UdpSocket::bind(("0.0.0.0", self.port)) {
            Ok(socket) => socket,
            Err(err) => {
                error!("Failed to start server: {}", err);

                std::process::exit(1);
            }
        };
        info!("Listening on 0.0.0.0:{}", self.port);

        let mut buf = [0; 512];
        let mut resolver = Resolver::new(
            UdpHandler::new(get_root_servers_handle_targets(false))
        );

        if let Some(forwards) = &self.forward {
            let mut targets = Vec::new();
            for target in forwards {
                targets.push(HandlerTarget::new(IpAddr::from_str(target.as_str())?, 53))
            }

            resolver = Resolver::new(UdpHandler::new(targets));
        }

        loop {
            match udp_socket.recv_from(&mut buf) {
                Ok((_size, source)) => {
                    let res = match resolver.resolve(&buf) {
                        Ok(res) => res,
                        Err(e) => {
                            error!("Resolve error: {}", e.to_string());

                            continue;
                        }
                    };
                    udp_socket.send_to(res.as_slice(), source).expect("Failed to send response");
                }
                Err(e) => {
                    error!("Error receiving data: {}", e);
                }
            }
        }
    }
}
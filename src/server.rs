use std::net::{SocketAddr, UdpSocket};
use anyhow::{Result};
use tracing::{error, info};
use crate::parser::PacketParser;
use crate::resolver::{ForwardResolver, RecursiveResolver, Resolver};

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
        let mut resolver: Box<dyn Resolver> = Box::new(RecursiveResolver::new());

        if let Some(forwards) = &self.forward {
            let mut targets = Vec::new();
            for target in forwards {
                targets.push(SocketAddr::new(target.parse()?, 53))
            }

            resolver = Box::new(ForwardResolver::new(targets));
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
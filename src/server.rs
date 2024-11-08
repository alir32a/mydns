use std::net::{UdpSocket};
use anyhow::{bail, Result};
use tracing::{error, info};
use crate::context::{Context, ListenerProtocol, ServerContext};
use crate::resolver::{AuthoritativeResolver, ForwardResolver, RecursiveResolver, Resolver};

pub trait DnsServer {
    fn start(&self) -> Result<()>;
}

pub struct UdpDnsServer {
    pub ctx: Context
}

impl UdpDnsServer {
    pub fn new(ctx: Context) -> UdpDnsServer {
        Self {
            ctx
        }
    }
}

impl DnsServer for UdpDnsServer {
    fn start(&self) -> Result<()> {
        let resolver: Box<dyn Resolver>;
        
        match &self.ctx.server {
            ServerContext::Authoritative { zones, nested_zones, .. } => {
                resolver = Box::new(AuthoritativeResolver::new(zones.clone(), *nested_zones)?);
            },
            ServerContext::Proxy { forward } => {
                resolver = Box::new(ForwardResolver::new(forward.to_vec()));
            },
            ServerContext::Recursive => {
                resolver = Box::new(RecursiveResolver::new());
            }
        }
        
        info!("Running in {} mode", self.ctx.server);

        self.run_server(resolver)
    }
}

impl UdpDnsServer {
    fn run_server(&self, mut resolver: Box<dyn Resolver>) -> Result<()> {
        let listener = &self.ctx.listener;

        match listener.proto {
            ListenerProtocol::UDP => {
                let udp_socket = match UdpSocket::bind(self.ctx.listener.to_addr()) {
                    Ok(socket) => socket,
                    Err(err) => {
                        error!("Failed to start server: {}", err);

                        std::process::exit(1);
                    }
                };
                info!("Listening on {}", self.ctx.listener);

                let mut buf = [0; 512];

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
                            error!("Error receiving records: {}", e);
                        }
                    }
                }
            },
            _ => {
                bail!("{} is not supported yet", listener.proto)
            }
        }
    }
}
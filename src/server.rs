use std::sync::Arc;
use tokio::net::{UdpSocket};
use anyhow::{bail, Result};
use tracing::{error, info};
use crate::context::{Context, ListenerProtocol, ServerMode};
use crate::resolver::{AuthoritativeResolver, ForwardResolver, RecursiveResolver, Resolver};

pub trait DnsServer {
    async fn start(&self) -> Result<()>;
}

pub struct UdpDnsServer {
    pub ctx: Arc<Context>
}

impl UdpDnsServer {
    pub fn new(ctx: Context) -> UdpDnsServer {
        Self {
            ctx: Arc::new(ctx)
        }
    }
}

impl DnsServer for UdpDnsServer {
    async fn start(&self) -> Result<()> {
        let resolver: Arc<Box<dyn Resolver + Send + Sync>>;

        match &self.ctx.server.mode {
            ServerMode::Authoritative { zones, nested_zones, .. } => {
                resolver = Arc::new(Box::new(AuthoritativeResolver::new(zones.clone(), *nested_zones)?));
            },
            ServerMode::Proxy { .. } => {
                resolver = Arc::new(Box::new(ForwardResolver::new(self.ctx.clone())));
            },
            ServerMode::Recursive { .. } => {
                resolver = Arc::new(Box::new(RecursiveResolver::new(self.ctx.clone())));
            }
        }
        
        info!("Running in {} mode", self.ctx.server.mode);

        let listener = &self.ctx.listener;

        match listener.proto {
            ListenerProtocol::UDP => {
                let udp_socket = match UdpSocket::bind(self.ctx.listener.to_addr()).await {
                    Ok(socket) => Arc::new(socket),
                    Err(err) => {
                        error!("Failed to start server: {}", err);

                        std::process::exit(1);
                    }
                };
                info!("Listening on {}", self.ctx.listener);

                let mut buf = vec![0; self.ctx.listener.max_packet_buf];
                loop {
                    match udp_socket.recv_from(&mut buf).await {
                        Ok((_size, source)) => {
                            let buf = Arc::new(buf.clone());
                            let udp_socket = udp_socket.clone();
                            let resolver = resolver.clone();
                            
                            tokio::spawn(async move {
                                let res = match resolver.resolve(buf) {
                                    Ok(res) => res,
                                    Err(e) => {
                                        error!("Resolve error: {}", e.to_string());

                                        return;
                                    }
                                };

                                udp_socket.
                                    send_to(res.as_slice(), source).
                                    await.
                                    expect("Failed to send response");
                            });
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
use std::fmt::{Display, Formatter};
use std::net::{IpAddr, SocketAddr};
use std::ops::Add;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use crate::Args;
use crate::cache::DnsCache;
use anyhow::{bail, Result};
use crate::handler::{HandlerStrategy, HandlerTarget};
use tokio::net::{ToSocketAddrs};

pub struct Context {
    pub(crate) cache: DnsCache,
    pub(crate) listener: ListenerContext,
    pub(crate) server: ServerContext,
    pub(crate) resolver: ResolverContext,
}

impl Context {
    pub fn from(args: Args) -> Result<Self> {
        let cache = DnsCache::new();
        let listener = ListenerContext::from(&args);

        if let Some(zones) = &args.zones {
            return Ok(
                Self {
                    cache,
                    listener,
                    server: ServerContext{
                        mode: ServerMode::Authoritative {
                            zones: PathBuf::from(zones),
                            nested_zones: args.nested_zones
                        },
                        ..Default::default()
                    },
                    resolver: Default::default()
                }
            )
        }
        
        if let Some(addrs) = args.forward {
            return Ok(
                Self {
                    cache,
                    listener,
                    server: ServerContext {
                        mode: ServerMode::Proxy {
                            forward: to_handler_targets(addrs, args.default_forward_port)?,
                            strategy: Default::default(), // you cannot set round-robin from args
                            default_port: args.default_forward_port,
                        },
                        ..Default::default()
                    },
                    resolver: Default::default()
                }
            )
        }
        
        Ok(
            Self {
                cache,
                listener,
                server: ServerContext {
                    ..Default::default()
                },
                resolver: Default::default()
            }
        )
    }
}

pub struct ServerContext {
    pub retry_interval: Duration,
    pub default_timeout: Duration,
    pub enable_ipv6: bool,
    pub mode: ServerMode
}

impl Default for ServerContext {
    fn default() -> Self {
        Self {
            retry_interval: Duration::from_secs(5),
            default_timeout: Duration::from_secs(3),
            enable_ipv6: false,
            mode: Default::default()
        }
    }
}

#[derive(Default)]
pub enum ServerMode {
    #[default]
    Recursive,
    Authoritative {
        zones: PathBuf,
        nested_zones: bool,
    },
    Proxy {
        forward: Vec<HandlerTarget>,
        strategy: HandlerStrategy,
        default_port: u16,
    }
}

impl Display for ServerMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerMode::Recursive => write!(f, "Recursive"),
            ServerMode::Authoritative { zones, .. } => {
                write!(f, "Authoritative (zones in {})", zones.display())
            },
            ServerMode::Proxy { forward, .. } => {
                write!(f, "Proxy (Forwarding to {})", join_addrs(forward, ", "))
            }
        }
    }
}

pub struct ListenerContext {
    pub port: u16,
    pub host: String,
    pub proto: ListenerProtocol,
    pub max_packet_buf: usize
}

impl ListenerContext {
    pub fn new(proto: ListenerProtocol, host: &str, port: u16, max_packet_buf: usize) -> Self {
        Self {
            host: host.to_string(),
            port,
            proto,
            max_packet_buf
        }
    }

    pub fn to_addr(&self) -> impl ToSocketAddrs + '_ {
        (self.host.as_str(), self.port)
    }

    fn from(args: &Args) -> Self {
        Self {
            host: args.host.to_owned(),
            port: args.port,
            proto: ListenerProtocol::from(&args.proto),
            max_packet_buf: 512
        }
    }
}

impl Display for ListenerContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:///{}:{}", self.proto, self.host, self.port)
    }
}

#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub enum ListenerProtocol {
    #[default]
    UDP,
    TCP
}

impl ListenerProtocol {
    pub fn from(proto: &String) -> Self {
        match proto.to_lowercase().as_str() {
            "tcp" => Self::TCP,
            _ => Self::UDP
        }
    }
}

impl Display for ListenerProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListenerProtocol::UDP => write!(f, "udp"),
            ListenerProtocol::TCP => write!(f, "tcp")
        }
    }
}

#[derive(Default, PartialEq, Eq, Clone)]
pub enum CacheContext {
    #[default]
    Internal,
    Redis {
        host: String,
        port: u16,
        password: String
    }
}

pub struct DatabaseContext {
    host: String,
    port: u16,
    user: String,
    password: String,
    db: String
}

pub struct ResolverContext {
    pub max_recursion_depth: usize,
    pub max_parse_jumps: usize
}

impl Default for ResolverContext {
    fn default() -> Self {
        Self {
            max_recursion_depth: 10,
            max_parse_jumps: 6
        }
    }
}

fn to_handler_targets(addrs: Vec<String>, default_port: u16) -> Result<Vec<HandlerTarget>> {
    let mut res = Vec::new();

    for addr in addrs {
        match SocketAddr::from_str(&addr) {
            Ok(addr) => {
                res.push(HandlerTarget::from_addr(addr))
            },
            Err(_e) => {
                match IpAddr::from_str(&addr) {
                    Ok(ip_addr) => {
                        res.push(HandlerTarget::from_addr(SocketAddr::new(ip_addr, default_port)))
                    },
                    Err(_e) => {
                        bail!("{} is not a valid address", addr)
                    }
                }
            }
        }
    }

    Ok(res)
}

fn join_addrs(targets: &Vec<HandlerTarget>, sep: &str) -> String {
    let mut res = String::new();

    for i in 0..targets.len() - 1 {
        res = res.add(&fmt_addr(&targets[i]).add(sep));
    }

    if let Some(addr) = targets.last() {
        res = res.add(&fmt_addr(&addr))
    }

    res
}

fn fmt_addr(target: &HandlerTarget) -> String {
    match target.addr.port() {
        53 => target.addr.ip().to_string(),
        _ => target.addr.to_string()
    }
}
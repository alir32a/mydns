use std::fmt::{Display, Formatter};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::ops::Add;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use crate::Args;
use crate::cache::DnsCache;
use anyhow::{bail, Result};
use crate::duration::parse;

pub struct Context {
    pub(crate) cache: DnsCache,
    pub(crate) listener: ListenerContext,
    pub(crate) server: ServerContext,
    pub(crate) timeout: Duration,
    pub(crate) use_ipv6: bool
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
                    server: ServerContext::Authoritative {
                        zones: PathBuf::from(zones),
                        nested_zones: args.nested_zones
                    },
                    use_ipv6: args.use_ipv6,
                    timeout: parse(&args.timeout)?
                }
            )
        }
        
        if let Some(addrs) = args.forward {
            return Ok(
                Self {
                    cache,
                    listener,
                    server: ServerContext::Proxy {
                        forward: to_socket_addrs(addrs)?
                    },
                    use_ipv6: args.use_ipv6,
                    timeout: parse(&args.timeout)?
                }
            )
        }
        
        Ok(
            Self {
                cache,
                listener,
                server: ServerContext::Recursive,
                use_ipv6: args.use_ipv6,
                timeout: parse(&args.timeout)?
            }
        )
    }
}

#[derive(Default)]
pub enum ServerContext {
    #[default]
    Recursive,
    Authoritative {
        zones: PathBuf,
        nested_zones: bool,
    },
    Proxy {
        forward: Vec<SocketAddr>,
    }
}

impl Display for ServerContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerContext::Recursive => write!(f, "Recursive"),
            ServerContext::Authoritative { zones, .. } => {
                write!(f, "Authoritative (zones in {})", zones.display())
            },
            ServerContext::Proxy {forward} => {
                write!(f, "Proxy (Forwarding to {})", join_addrs(forward, ", "))
            }
        }
    }
}

pub struct ListenerContext {
    pub port: u16,
    pub host: String,
    pub proto: ListenerProtocol
}

impl ListenerContext {
    pub fn new(proto: ListenerProtocol, host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            proto
        }
    }

    pub fn to_addr(&self) -> impl ToSocketAddrs + '_ {
        (self.host.as_str(), self.port)
    }

    fn from(args: &Args) -> Self {
        Self {
            host: args.host.to_owned(),
            port: args.port,
            proto: ListenerProtocol::from(&args.proto)
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
            "udp" => Self::UDP,
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

fn to_socket_addrs(addrs: Vec<String>) -> Result<Vec<SocketAddr>> {
    let mut res = Vec::new();

    for addr in addrs {
        match SocketAddr::from_str(&addr) {
            Ok(addr) => res.push(addr),
            Err(_e) => {
                match IpAddr::from_str(&addr) {
                    Ok(ip_addr) => {
                        res.push(SocketAddr::new(ip_addr, 53))
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

fn join_addrs(addrs: &Vec<SocketAddr>, sep: &str) -> String {
    let mut res = String::new();

    for i in 0..addrs.len() - 1 {
        res = res.add(&fmt_addr(&addrs[i]).add(sep));
    }

    if let Some(addr) = addrs.last() {
        res = res.add(&fmt_addr(&addr))
    }

    res
}

fn fmt_addr(addr: &SocketAddr) -> String {
    match addr.port() {
        53 => addr.ip().to_string(),
        _ => addr.to_string()
    }
}
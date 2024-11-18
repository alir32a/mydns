use std::fmt::{Display, Formatter};
use std::net::{IpAddr, SocketAddr};
use std::ops::{Add};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use crate::Args;
use crate::cache::DnsCache;
use anyhow::{bail, Result};
use crate::handler::{HandlerStrategy, HandlerTarget};
use tokio::net::{ToSocketAddrs};
use tracing::error;
use crate::config::{load_config, Config, ForwardAddr, Mode};
use crate::duration::parse;

pub struct Context {
    pub(crate) cache: DnsCache,
    pub(crate) listener: ListenerContext,
    pub(crate) server: ServerContext,
    pub(crate) resolver: ResolverContext,
}

impl Context {
    pub fn from(args: Args) -> Result<Self> {
        let mut cfg = Self::load_config(&args.config_file);
        cfg = cfg.apply_args(args);
        
        let mode = Self::get_server_mode(&cfg)?;
        let proto = ListenerProtocol::from(cfg.listener.proto.unwrap_or_default());

        Ok(Self {
            cache: DnsCache::new(),
            listener: ListenerContext {
                host: cfg.listener.host.unwrap_or("0.0.0.0".to_string()),
                port: cfg.listener.port.unwrap_or(53),
                proto,
                max_packet_buf: cfg.listener.max_packet_buf.unwrap_or(512)
            },
            server: ServerContext {
                retry_interval: parse(&cfg.server.retry_interval.unwrap_or("5s".to_string()))?,
                default_timeout: parse(&cfg.server.default_timeout.unwrap_or("3s".to_string()))?,
                enable_ipv6: cfg.server.enable_ipv6.unwrap_or_default(),
                mode,
            },
            resolver: ResolverContext {
                max_recursion_depth: cfg.resolver.max_recursion_depth.unwrap_or(10),
                max_parse_jumps: cfg.resolver.max_parse_jumps.unwrap_or(6)
            }
        })
    }
    
    fn load_config(path: &Option<String>) -> Config {
        match load_config(path.clone()) {
            Ok(mut cfg) => {
                if cfg.server.authoritative.is_some() {
                    cfg.mode = Mode::AUTHORITATIVE;
                }
                
                if cfg.server.forward.is_some() {
                    cfg.mode = Mode::PROXY
                }
                
                cfg
            },
            Err(e) => {
                if let Some(p) = path {
                    error!("couldn't load {} file, {}", p, e.to_string())
                }
                
                Default::default()
            }
        }
    }
    
    fn get_server_mode(cfg: &Config) -> Result<ServerMode> {
        match cfg.mode { 
            Mode::RECURSIVE => Ok(ServerMode::Recursive),
            Mode::AUTHORITATIVE => {
                Ok(ServerMode::Authoritative {
                    zones: cfg.server.authoritative.clone().unwrap_or_default().zones.unwrap_or_default(),
                    nested_zones: cfg.server.authoritative.clone().unwrap_or_default().nested_zones.unwrap_or_default(),
                })
            },
            Mode::PROXY => {
                match &cfg.server.forward { 
                    Some(forward) => {
                        Ok(ServerMode::Proxy {
                            forward: Self::get_handler_targets(&forward.addrs, forward.default_port.unwrap_or(53))?,
                            strategy: HandlerStrategy::from(&forward.strategy.clone().unwrap_or_default()),
                            default_port: forward.default_port.unwrap_or(53),
                        })
                    },
                    None => bail!("forward addresses are empty, nowhere to forward")
                }
            },
        }
    }
    
    fn get_handler_targets(addrs: &Vec<ForwardAddr>, default_port: u16) -> Result<Vec<HandlerTarget>> {
        let targets = addrs.iter().map(|addr| {
            HandlerTarget::new(&addr.addr, default_port, addr.weight.unwrap_or_default())
        }).collect::<Result<Vec<HandlerTarget>>>()?;

        Ok(targets)
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
}

impl Display for ListenerContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
    pub fn from(proto: String) -> Self {
        match proto.to_lowercase().as_str() {
            "tcp" => Self::TCP,
            _ => Self::UDP
        }
    }
}

impl Display for ListenerProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
    path: PathBuf
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
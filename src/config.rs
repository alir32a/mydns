use std::path::{PathBuf};
use anyhow::{bail, Result};
use serde::Deserialize;
use tracing::warn;
use crate::args::Args;
use crate::fs::{check_home_dir, get_home_dir};

#[derive(Default, Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub listener: ListenerConfig,
    #[serde(default)]
    pub resolver: ResolverConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(skip_deserializing)]
    pub mode: Mode,
}

impl Config {
    pub fn apply_args(mut self, args: Args) -> Self {
        self.listener.port = args.port.or(self.listener.port);
        self.listener.host = args.host.or(self.listener.host);
        self.listener.proto = args.proto.or(self.listener.proto);
        self.server.default_timeout = args.timeout.or(self.server.default_timeout);
        self.server.enable_ipv6 = args.enable_ipv6.or(self.server.enable_ipv6);
        
        if args.zones.is_some() || args.nested_zones.is_some() {
            self.server.authoritative = match self.server.authoritative {
                Some(mut authoritative) => {
                    authoritative.zones = args.zones.map(|zones| {
                        PathBuf::from(zones)
                    }).or(authoritative.zones);
                    authoritative.nested_zones = args.nested_zones.or(authoritative.nested_zones);
                    
                    Some(authoritative)
                }, None => {
                    Some(Authoritative {
                        zones: args.zones.map(|zones| {
                            PathBuf::from(zones)
                        }),
                        nested_zones: args.nested_zones,
                    })
                }
            };
        }
        
        if let Some(addrs) = args.forward {
            self.server.forward = Some(Forward {
                addrs: addrs.into_iter().map(|addr| {
                    ForwardAddr::from(addr)
                }).collect(),
                default_port: args.default_forward_port.or(self.server.forward.and_then(|forward| {
                   forward.default_port 
                })),
                ..Default::default()
            });
            
            self.mode = Mode::PROXY;
        }
        
        if args.authoritative && self.server.authoritative.is_none() {
            match get_home_dir() {
                Some(path) => {
                    self.server.authoritative = Some(Authoritative {
                        zones: Some(path.join("zones")),
                        nested_zones: args.nested_zones,
                    });
                },
                None => warn!("no zones provided, if there aren't any in database either, you are not serving any zone!")
            };
            
            self.mode = Mode::AUTHORITATIVE
        }
        
        self
    }
}

#[derive(Default, Deserialize, Debug)]
pub struct ListenerConfig {
    pub port: Option<u16>,
    pub host: Option<String>,
    pub proto: Option<String>,
    pub max_packet_buf: Option<usize>
}

#[derive(Default, Deserialize, Debug)]
pub struct ServerConfig {
    pub retry_interval: Option<String>,
    pub default_timeout: Option<String>,
    pub enable_ipv6: Option<bool>,
    pub authoritative: Option<Authoritative>,
    pub forward: Option<Forward>
}

#[derive(Default, Clone, Deserialize, Debug)]
pub struct Authoritative {
    pub zones: Option<PathBuf>,
    pub nested_zones: Option<bool>
}

#[derive(Default, Deserialize, Debug)]
pub struct Forward {
    pub addrs: Vec<ForwardAddr>,
    pub strategy: Option<String>,
    pub default_port: Option<u16>,
}

#[derive(Deserialize, Debug)]
pub struct ForwardAddr {
    pub addr: String,
    pub weight: Option<usize>
}

impl ForwardAddr {
    pub fn from(addr: String) -> Self {
        Self {
            addr,
            weight: Some(1),
        }
    }
}

#[derive(Default, Deserialize, Debug)]
pub struct ResolverConfig {
    pub max_recursion_depth: Option<usize>,
    pub max_parse_jumps: Option<usize>
}

#[derive(Default, Deserialize, Debug)]
pub enum Mode {
    #[default]
    RECURSIVE,
    AUTHORITATIVE,
    PROXY
}

pub fn load_config(path: Option<String>) -> Result<Config> {
    if let Some(path) = path {
        return load(PathBuf::from(path));
    }
    
    check_home_dir();
    
    if let Some(path) = get_home_dir() {
        return load(PathBuf::from(path).join("conf.toml"));
    }
    
    bail!("couldn't find any config file")
}

fn load(p: PathBuf) -> Result<Config> {
    let file = std::fs::read_to_string(p)?;
    
    let cfg: Config = toml::from_str(&file)?;

    Ok(cfg)
}
use std::io::{Error, ErrorKind};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use rand::{thread_rng, Rng};
use anyhow::{bail, Result};
use log::info;
use serde::Deserialize;
use tokio::{
    select,
    time::{
        Duration
    },
    sync::{
        mpsc
    }
};
use tracing::error;
use crate::context::{Context, ServerMode};
use crate::packet::Packet;
use crate::root::get_root_servers_socket_addrs;
use crate::writer::PacketWriter;

pub trait Handler {
    fn send(&self, buf: &[u8]) -> Result<Vec<u8>>;
    fn send_to(&self, buf: &[u8], addrs: &[SocketAddr]) -> Result<Vec<u8>>;
}

struct Zero;

pub struct UdpHandler {
    ctx: Arc<Context>,
    socket: Arc<UdpSocket>,
    targets: Arc<RwLock<Box<dyn HandlerQueue>>>,
    failures: Arc<RwLock<Vec<HandlerTarget>>>,
    shutdown_fn: Arc<mpsc::Sender<Zero>>,
}

impl UdpHandler {
    pub fn try_new(ctx: Arc<Context>) -> Result<Self> {
        let socket = UdpSocket::bind(
            ("0.0.0.0", thread_rng().gen_range(9999..u16::MAX))
        )?;
        socket.set_read_timeout(Some(ctx.server.default_timeout))?;
        socket.set_write_timeout(Some(ctx.server.default_timeout))?;

        let (tx, rx) = mpsc::channel(1);

        let mut handler = Self {
            ctx: ctx.clone(),
            socket: Arc::new(socket),
            targets: Arc::new(RwLock::new(get_queue(ctx.clone())?)),
            failures: Arc::new(RwLock::new(Vec::new())),
            shutdown_fn: Arc::new(tx)
        };

        handler.run_failures_job(rx);
        
        Ok(handler)
    }

    pub fn new(ctx: Arc<Context>) -> Self {
        Self::try_new(ctx).unwrap()
    }

    fn run_failures_job(&mut self, mut shutdown: mpsc::Receiver<Zero>) {
        let socket = self.socket.clone();
        let failures = self.failures.clone();
        let targets = self.targets.clone();
        let retry_interval = self.ctx.server.retry_interval;

         tokio::spawn(async move {
            let mut interval = tokio::time::interval(retry_interval);

            loop {
                select! {
                    _ = interval.tick() => {
                        let faileds = failures.read().unwrap().to_vec();

                        for (i, target) in faileds.into_iter().enumerate() {
                            info!("retrying {}...", target.addr);
                            
                            let buf = PacketWriter::from(Packet::get_empty_packet()).write().unwrap();

                            match socket.send_to(buf.as_slice(), target.addr) {
                                Ok(n) => {
                                    if n == 0 {
                                        error!("Got an error while retrying {}", target.addr.to_string());

                                        continue;
                                    }

                                    let mut buf = [0u8; 512];
                                    match socket.recv(&mut buf) {
                                        Ok(_n) => {
                                            failures.write().unwrap().remove(i);  
                                            targets.write().unwrap().push(target);
                                        },
                                        Err(e) => {
                                            error!(
                                                "Got an error while retrying \
                                                {} => {}",
                                                target.addr.to_string(),
                                                e.to_string(),
                                            );
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!(
                                        "Got an error while retrying {} => {}",
                                        target.addr.to_string(),
                                        e.to_string(),
                                    );
                                }
                            }
                        }
                    },
                    _ = shutdown.recv() => {
                        break;
                    }
                }
            }
        });
    }
}

impl Handler for UdpHandler {
    fn send(&self, buf: &[u8]) -> Result<Vec<u8>> {
        let mut res = [0u8; 512];
        let mut sent = false;
        let mut queue = self.targets.write().expect("");
        
        while let Some(target) = queue.fetch() {
            if let Err(e) = self.socket.send_to(buf, target.addr) {
                if is_timeout(&e) {
                    if let Some(target) = queue.remove() {
                        self.failures.write().unwrap().push(target);
                    }
                    
                    error!("{} not responding, moving to the next resource", target.addr);

                    continue;
                }

                bail!(e);
            }

            if let Err(e) = self.socket.recv_from(&mut res) {
                if is_timeout(&e) {
                    if let Some(target) = queue.remove() {
                        self.failures.write().unwrap().push(target);
                    }

                    error!("{} not responding, moving to the next resource", target.addr);

                    continue;
                }

                bail!(e);
            }
            
            sent = true;
            break
        }

        if !sent {
            bail!("all of the given addresses failed to serve the request")
        }

        Ok(res.to_vec())
    }

    fn send_to(&self, buf: &[u8], addrs: &[SocketAddr]) -> Result<Vec<u8>> {
        let mut res = vec![0; 512];
        let mut sent = false;
        
        for addr in addrs {
            if addr.is_ipv6() && !self.ctx.server.enable_ipv6 {
                continue
            }
            
            if let Err(e) = self.socket.send_to(buf, addr) {
                if is_timeout(&e) {
                    continue;
                }

                bail!(e);
            }

            if let Err(e) = self.socket.recv_from(&mut res) {
                if is_timeout(&e) {
                    continue;
                }

                bail!(e);
            }

            if res.is_empty() {
                bail!("Got empty response from {}", addr.to_string());
            }
            
            sent = true;
            break;
        }

        if !sent {
            bail!("all of the given addresses failed to serve the request")
        }
        
        Ok(res)
    }
}

impl Drop for UdpHandler {
    fn drop(&mut self) {
        self.shutdown_fn.try_send(Zero).unwrap_or_else(|err| {
            error!("Couldn't shutdown one of the handlers task: {}", err);
        })
    }
}

#[derive(Default, PartialEq, Eq, Copy, Clone, Deserialize)]
pub enum HandlerStrategy {
    #[default]
    Standard,
    RoundRobin
}

impl HandlerStrategy {
    pub fn from(s: &str) -> Self {
        match s { 
            "round-robin" => Self::RoundRobin,
            _ => Self::Standard
        }
    }
}

#[derive(Copy, Clone, Deserialize)]
pub struct HandlerTarget {
    pub addr: SocketAddr,
    pub weight: usize,
}

impl HandlerTarget {
    pub fn new(addr: &str, default_port: u16, weight: usize) -> Result<Self> {
        let socket_addr: SocketAddr;
        
        match SocketAddr::from_str(addr) { 
            Ok(addr) => {
                socket_addr = addr;
            },
            Err(_e) => {
                socket_addr = SocketAddr::new(IpAddr::from_str(addr)?, default_port)
            }
        }
        
        Ok(Self {
            addr: socket_addr,
            weight
        })
    }
    
    pub fn from_addr(addr: SocketAddr) -> Self {
        Self {
            addr,
            weight: 1,
        }
    }
}

pub trait HandlerQueue: Send + Sync {
    fn fetch(&mut self) -> Option<HandlerTarget>;
    fn next(&mut self) -> Option<HandlerTarget>;
    fn push(&mut self, target: HandlerTarget);
    fn remove(&mut self) -> Option<HandlerTarget>;
}

pub struct StandardQueue {
    targets: Vec<HandlerTarget>,
    offset: usize
}

impl StandardQueue {
    pub fn new(targets: Vec<HandlerTarget>) -> Self {
        Self {
            targets,
            offset: 0
        }
    }
}

impl HandlerQueue for StandardQueue {
    fn fetch(&mut self) -> Option<HandlerTarget> {
        if let Some(target) = self.targets.get(self.offset) {
            return Some(target.clone());
        }

        // reset offset if we are at the end of the queue
        self.offset = 0;
        if self.targets.len() > 0 {
            return self.fetch();
        }

        None
    }
    
    fn next(&mut self) -> Option<HandlerTarget> {
        self.offset += 1;
        
        self.fetch()
    }

    fn push(&mut self, target: HandlerTarget) {
        self.targets.push(target);
    }
    
    fn remove(&mut self) -> Option<HandlerTarget> {
        if self.offset >= self.targets.len() {
            return None;
        }
        
        let target = self.targets.remove(self.offset);
        self.offset += 1;
        
        Some(target)
    }
}

pub struct RoundRobinQueue {
    targets: Vec<HandlerTarget>,
    offset: usize,
    counter: usize
}

impl RoundRobinQueue {
    pub fn new(targets: Vec<HandlerTarget>) -> Self {
        Self {
            targets,
            offset: 0,
            counter: 0
        }
    }
}

impl HandlerQueue for RoundRobinQueue {
    fn fetch(&mut self) -> Option<HandlerTarget> {
        if let Some(target) = self.targets.get(self.offset) {
            self.counter += 1;

            if self.counter == target.weight {
                self.offset += 1;
                self.counter = 0;
            }

            return Some(target.clone());
        }

        // reset offset if we are at the end of the queue
        self.offset = 0;
        if self.targets.len() > 0 {
            return self.fetch();
        }

        None
    }
    
    fn next(&mut self) -> Option<HandlerTarget> {
        self.offset += 1;
        self.counter = 0;
        
        self.fetch()
    }

    fn push(&mut self, target: HandlerTarget) {
        self.targets.push(target)
    }

    fn remove(&mut self) -> Option<HandlerTarget> {
        if self.offset >= self.targets.len() {
            return None;
        }

        let target = self.targets.remove(self.offset);
        self.offset += 1;

        Some(target)
    }
}

fn is_timeout(err: &Error) -> bool {
    err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut
}

fn get_queue(ctx: Arc<Context>) -> Result<Box<dyn HandlerQueue>> {
    let mut targets = Vec::new();

    match &ctx.server.mode {
        ServerMode::Proxy { forward, strategy, .. } => {
            forward.iter().for_each(|target| {
                if target.addr.is_ipv6() && !ctx.server.enable_ipv6 {
                    return;
                }
                
                targets.push(target.clone());
            });
            
            match strategy {
                HandlerStrategy::Standard => Ok(Box::new(StandardQueue::new(targets))),
                HandlerStrategy::RoundRobin => Ok(Box::new(RoundRobinQueue::new(targets)))
            }
        },
        ServerMode::Recursive => {
            targets.append(&mut get_root_servers_socket_addrs(ctx.server.enable_ipv6));
            
            Ok(Box::new(StandardQueue::new(targets)))
        },
        _ => bail!("{} is not supposed to use a handler", ctx.server.mode),
    }
}
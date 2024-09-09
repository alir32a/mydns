use std::cell::RefCell;
use std::io::{Error, ErrorKind};
use std::net::{AddrParseError, IpAddr, ToSocketAddrs, UdpSocket};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use rand::{thread_rng, Rng};
use anyhow::{bail, Result};
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
use crate::packet::Packet;
use crate::writer::PacketWriter;

pub trait Handler {
    fn send(&mut self, buf: &[u8]) -> Result<Vec<u8>>;
}

#[derive(Clone)]
pub struct HandlerTarget {
    pub addr: IpAddr,
    pub port: u16,
}

impl HandlerTarget {
    pub fn new(addr: IpAddr, port: u16) -> Self {
        Self {
            addr,
            port
        }
    }
}

struct Zero;

pub struct UdpHandler {
    failures: Arc<RwLock<Vec<usize>>>,
    targets: Arc<Vec<HandlerTarget>>,
    shutdown_fn: Arc<mpsc::Sender<Zero>>
}

impl UdpHandler {
    pub fn new(addrs: Vec<HandlerTarget>) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let mut handler = Self {
            failures: Arc::new(RwLock::new(Vec::new())),
            targets: Arc::new(Vec::from(addrs)),
            shutdown_fn: Arc::new(tx)
        };

        handler.run_failures_job(rx);

        handler
    }

    fn run_failures_job(&mut self, mut shutdown: mpsc::Receiver<Zero>) {
        let failures = self.failures.clone();
        let queue = self.targets.clone();

         tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));

            let socket = UdpSocket::bind(
                ("0.0.0.0", thread_rng().gen_range(9999..u16::MAX))
            ).unwrap();
            socket.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            socket.set_write_timeout(Some(Duration::from_secs(5))).unwrap();

            loop {
                select! {
                    _ = interval.tick() => {
                        let faileds = failures.read().unwrap().to_vec();

                        for (i, idx) in faileds.into_iter().enumerate() {
                            let buf = PacketWriter::from(Packet::get_empty_packet()).write().unwrap();
                            let target = queue[idx].clone();

                            match socket.send_to(buf.as_slice(), (target.addr, target.port)) {
                                Ok(n) => {
                                    if n == 0 {
                                        error!("Unable to send UDP message to {}:{}", target.addr, target.port);

                                        continue;
                                    }

                                    let mut buf = [0u8; 512];
                                    match socket.recv(&mut buf) {
                                        Ok(n) => {
                                            failures.write().unwrap().remove(i);
                                        },
                                        Err(e) => {
                                            error!(
                                                "Got an error while receiving UDP message to \
                                                {}:{} => {}",
                                                target.addr,
                                                target.port,
                                                e.to_string(),
                                            );
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!(
                                        "Got an error while sending UDP message to {}:{} => {}",
                                        target.addr,
                                        target.port,
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
    fn send(&mut self, buf: &[u8]) -> Result<Vec<u8>> {
        let mut res = [0u8; 512];
        let mut sent = false;

        for (i, target) in self.targets.iter().enumerate() {
            if self.failures.read().unwrap().len() == self.targets.len() {
                bail!("all of the resolve targets are unavailable")
            }

            if self.failures.read().unwrap().contains(&i) {
                continue;
            }

            let socket = UdpSocket::bind(
                ("0.0.0.0", thread_rng().gen_range(9999..u16::MAX))
            )?;
            socket.set_write_timeout(Some(Duration::from_secs(5)))?;
            socket.set_read_timeout(Some(Duration::from_secs(5)))?;
            if let Err(e) = socket.send_to(buf, (target.addr, target.port)) {
                if is_timeout(&e) {
                    self.failures.write().unwrap().push(i);

                    continue;
                }

                bail!(e);
            }

            if let Err(e) = socket.recv_from(&mut res) {
                if is_timeout(&e) {
                    self.failures.write().unwrap().push(i);

                    continue;
                }

                bail!(e);
            }

            sent = true;
            break;
        }

        if !sent {
            bail!("Couldn't send request to any of the given addresses")
        }

        Ok(res.to_vec())
    }
}

impl Drop for UdpHandler {
    fn drop(&mut self) {

    }
}

fn is_timeout(err: &Error) -> bool {
    err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut
}
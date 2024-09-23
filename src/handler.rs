use std::io::{Error, ErrorKind};
use std::net::{IpAddr, SocketAddr, UdpSocket};
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
    fn send_to(&mut self, buf: &[u8], addrs: &[SocketAddr]) -> Result<Vec<u8>>;
}

struct Zero;

pub struct UdpHandler {
    socket: Arc<UdpSocket>,
    targets: Arc<Vec<SocketAddr>>,
    failures: Arc<RwLock<Vec<usize>>>,
    shutdown_fn: Arc<mpsc::Sender<Zero>>,
}

impl UdpHandler {
    pub fn try_new(addrs: Vec<SocketAddr>, timeout: Duration) -> Result<Self> {
        let socket = UdpSocket::bind(
            ("0.0.0.0", thread_rng().gen_range(9999..u16::MAX))
        )?;
        socket.set_read_timeout(Some(timeout))?;
        socket.set_write_timeout(Some(timeout))?;

        let (tx, rx) = mpsc::channel(1);

        let mut handler = Self {
            socket: Arc::new(socket),
            targets: Arc::new(Vec::from(addrs)),
            failures: Arc::new(RwLock::new(Vec::new())),
            shutdown_fn: Arc::new(tx)
        };

        handler.run_failures_job(rx);
        
        Ok(handler)
    }

    pub fn new(addrs: Vec<SocketAddr>, timeout: Duration) -> Self {
        Self::try_new(addrs, timeout).unwrap()
    }

    fn run_failures_job(&mut self, mut shutdown: mpsc::Receiver<Zero>) {
        let failures = self.failures.clone();
        let queue = self.targets.clone();
        let socket = self.socket.clone();

         tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                select! {
                    _ = interval.tick() => {
                        let faileds = failures.read().unwrap().to_vec();

                        for (i, idx) in faileds.into_iter().enumerate() {
                            let buf = PacketWriter::from(Packet::get_empty_packet()).write().unwrap();
                            let target = queue[idx].clone();

                            match socket.send_to(buf.as_slice(), target) {
                                Ok(n) => {
                                    if n == 0 {
                                        error!("Unable to send UDP message to {}", target.to_string());

                                        continue;
                                    }

                                    let mut buf = [0u8; 512];
                                    match socket.recv(&mut buf) {
                                        Ok(n) => {
                                            if let Some(n) = failures.read().unwrap().iter().find(|index| {
                                                **index == idx
                                            }) {
                                                failures.write().unwrap().remove(*n);   
                                            }
                                        },
                                        Err(e) => {
                                            error!(
                                                "Got an error while receiving UDP message to \
                                                {} => {}",
                                                target.to_string(),
                                                e.to_string(),
                                            );
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!(
                                        "Got an error while sending UDP message to {} => {}",
                                        target.to_string(),
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
        let mut timed_outs = Vec::with_capacity(self.targets.len());
        let mut sent = false;

        for (i, target) in self.targets.iter().enumerate() {
            if self.failures.read().expect("failures lock may be poisoned").len() == self.targets.len() {
                bail!("all of the resolve targets are unavailable")
            }

            if self.failures.read().expect("failures lock may be poisoned").contains(&i) {
                continue;
            }

            if let Err(e) = self.socket.send_to(buf, target) {
                if is_timeout(&e) {
                    timed_outs.push(i);

                    continue;
                }

                bail!(e);
            }

            if let Err(e) = self.socket.recv_from(&mut res) {
                if is_timeout(&e) {
                    timed_outs.push(i);

                    continue;
                }

                bail!(e);
            }

            sent = true;
            break;
        }

        self.failures.write().expect("failures lock may be poisoned").append(&mut timed_outs);

        if !sent {
            bail!("Couldn't send request to any of the given addresses")
        }

        Ok(res.to_vec())
    }

    fn send_to(&mut self, buf: &[u8], addrs: &[SocketAddr]) -> Result<Vec<u8>> {
        let mut res = vec![0; 512];
        let mut sent = false;
        
        for addr in addrs {
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
            bail!("Couldn't send request to any of the given addresses")
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

fn is_timeout(err: &Error) -> bool {
    err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut
}
use std::io::BufReader;
use std::net::UdpSocket;
use anyhow::{Result};
use rand::random;

pub struct StubResolver<'r> {
    pub forward: &'r str,
}

impl<'r> StubResolver<'r> {
    pub fn new(forward: &'r str) -> StubResolver {
        Self {
            forward,
        }
    }

    pub fn resolve(&self, req: &[u8]) -> Result<Vec<u8>> {
        let mut res = [0; 512];

        let socket = UdpSocket::bind(("0.0.0.0", random()))?;
        socket.send_to(req, (self.forward, 53))?;
        socket.recv_from(&mut res)?;

        Ok(res.to_vec())
    }
}
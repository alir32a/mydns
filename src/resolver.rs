use std::net::UdpSocket;
use anyhow::{Result};
use rand::{Rng, thread_rng};
use crate::packet::Packet;
use crate::parser::PacketParser;
use crate::result_code::ResultCode;
use crate::writer::PacketWriter;

pub struct Resolver<'r> {
    pub forward: &'r str,
}

impl<'r> Resolver<'r> {
    pub fn new(forward: &'r str) -> Resolver {
        Self {
            forward,
        }
    }

    pub fn resolve(&self, buf: &[u8]) -> Result<Vec<u8>> {
        let mut req = PacketParser::new(buf).parse()?;

        if req.questions.len() == 0 {
            let mut res = Packet::from(req);
            Self::set_resp_packet(&mut res);
            res.header.code = ResultCode::FORMERR.to_u8();

            return PacketWriter::from(res).write();
        }

        let socket = UdpSocket::bind(
            ("0.0.0.0", thread_rng().gen_range(9999..u16::MAX))
        )?;
        socket.send_to(buf, (self.forward, 53))?;

        let mut res_buf = [0; 512];
        socket.recv_from(&mut res_buf)?;

        let mut res = PacketParser::new(&res_buf).parse()?;
        Self::set_resp_packet(&mut res);

        PacketWriter::from(res).write()
    }

    fn set_resp_packet(res: &mut Packet) {
        res.header.recursion_desired = true;
        res.header.recursion_available = true;
        res.header.response = true;
    }
}
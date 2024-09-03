use std::net::UdpSocket;
use std::ops::Deref;
use anyhow::{Result};
use rand::{random, Rng, thread_rng};
use rand::rngs::ThreadRng;
use crate::packet::Packet;
use crate::parser::PacketParser;
use crate::question::Question;
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
        let mut res = Packet::from(&req);
        Self::set_resp_packet(&mut res);

        if req.questions.len() == 0 {
            let mut res = Packet::from(&req);
            Self::set_resp_packet(&mut res);
            res.header.code = ResultCode::FORMERR.to_u8();

            return PacketWriter::from(res).write();
        }

        let mut rng = thread_rng();
        for question in req.questions {
            if let Ok(result) = self.lookup(question, &mut rng) {
                Self::append_results(&mut res, result);
            } else {
                res.header.code = ResultCode::SERVFAIL.to_u8();

                break;
            }
        }

        PacketWriter::from(res).write()
    }

    pub fn lookup(&self, question: Question, rng: &mut ThreadRng) -> Result<Packet> {
        let req = Self::new_query_packet(question, rng);

        let req_bin = PacketWriter::from(req).write()?;

        let socket = UdpSocket::bind(
            ("0.0.0.0", rng.gen_range(9999..u16::MAX))
        )?;
        socket.send_to(req_bin.as_slice(), (self.forward, 53))?;

        let mut res_buf = [0; 512];
        socket.recv_from(&mut res_buf)?;

        let mut res = PacketParser::new(&res_buf).parse()?;
        Self::set_resp_packet(&mut res);

        Ok(res)
    }

    fn append_results(dst: &mut Packet, src: Packet) {
        dst.header.answer_count += src.header.answer_count;
        dst.header.authority_count += src.header.authority_count;
        dst.header.resource_count += src.header.resource_count;

        for answer in src.answers {
            dst.answers.push(answer);
        }

        for authority in src.authorities {
            dst.authorities.push(authority);
        }

        for resource in src.resources {
            dst.resources.push(resource);
        }
    }

    fn set_resp_packet(res: &mut Packet) {
        res.header.recursion_available = true;
        res.header.response = true;
    }

    fn new_query_packet(question: Question, rng: &mut ThreadRng) -> Packet {
        let mut req = Packet::new();
        req.header.id = rng.gen();
        req.header.recursion_desired = true;
        req.header.question_count = 1;
        req.questions.push(question);

        req
    }
}
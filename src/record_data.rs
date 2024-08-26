use std::net::{Ipv4Addr, Ipv6Addr};
use crate::pair::BytesPair;

#[derive(Debug)]
pub enum RecordData {
    A(Ipv4Addr),
    NS(String),
    CNAME(String),
    MX {
        preference: u16,
        exchange: String
    },
    AAAA(Ipv6Addr),
    ASTERISK(Box<[u8]>),
    UNKNOWN(u16)
}

impl RecordData {
    pub fn bytes(&self) -> Vec<u8> {
        match self {
            RecordData::A(addr) => {
                addr.octets().to_vec()
            },
            RecordData::NS(host) | RecordData::CNAME(host) => {
                host.bytes().collect()
            },
            RecordData::MX { preference, exchange } => {
                let mut res: Vec<u8> = BytesPair::from(*preference).bytes();
                res.append(&mut exchange.bytes().collect());

                res
            },
            RecordData::AAAA(addr) => {
                addr.octets().to_vec()
            },
            RecordData::ASTERISK(data) => {
                data.to_vec()
            },
            RecordData::UNKNOWN(n) => {
                vec![0; *n as usize]
            }
        }
    }
}
use std::net::{Ipv4Addr, Ipv6Addr};
use crate::bytes_util::BytesUtil;
use crate::pair::BytesPair;

#[derive(Debug)]
pub enum RecordData {
    A(Ipv4Addr),
    NS(String),
    CNAME(String),
    SOA {
        mname: String,
        rname: String,
        serial: u32,
        refresh: u32,
        retry: u32,
        expire: u32,
        minimum: u32
    },
    MB(String),
    MG(String),
    MR(String),
    NULL(Vec<u8>),
    WKS {
        addr: Ipv4Addr,
        protocol: u8,
        bit_map :Vec<u8>
    },
    PTR(String),
    HINFO {
        cpu: String,
        os: String
    },
    MINFO {
        rmailbox: String,
        emailbox: String
    },
    MX {
        preference: u16,
        exchange: String
    },
    TXT(String),
    AAAA(Ipv6Addr),
    UNKNOWN(u16)
}

impl RecordData {
    pub fn bytes(&self) -> Vec<u8> {
        match self {
            RecordData::A(addr) => {
                addr.octets().to_vec()
            },
            RecordData::NS(host) | RecordData::CNAME(host) |
            RecordData::MB(host) | RecordData::MG(host) |
            RecordData::MR(host)| RecordData::PTR(host) |
            RecordData::TXT(host) => {
                host.bytes().collect()
            },
            RecordData::SOA {
                mname,
                rname,
                serial,
                refresh,
                retry,
                expire,
                minimum
            } => {
                let mut res: Vec<u8> = mname.bytes().collect();
                res.append(&mut rname.bytes().collect());
                res.append(&mut BytesUtil::from_u32(*serial).to_vec());
                res.append(&mut BytesUtil::from_u32(*refresh).to_vec());
                res.append(&mut BytesUtil::from_u32(*retry).to_vec());
                res.append(&mut BytesUtil::from_u32(*expire).to_vec());
                res.append(&mut BytesUtil::from_u32(*minimum).to_vec());

                res
            },
            RecordData::NULL(data) => {
                data.clone()
            },
            RecordData::WKS { ref addr, protocol, ref bit_map } => {
                let mut res: Vec<u8> = vec![];
                res.append(&mut addr.octets().to_vec());
                res.push(*protocol);
                bit_map.iter().for_each(|byte| {
                    res.push(*byte);
                });

                res
            },
            RecordData::HINFO { ref cpu, ref os} => {
                let mut res: Vec<u8> = cpu.bytes().collect();
                res.append(&mut os.bytes().collect());

                res
            },
            RecordData::MINFO { ref rmailbox, ref emailbox} => {
                let mut res: Vec<u8> = rmailbox.bytes().collect();
                res.append(&mut emailbox.bytes().collect());

                res
            },
            RecordData::MX { preference, exchange } => {
                let mut res: Vec<u8> = BytesPair::from(*preference).bytes();
                res.append(&mut exchange.bytes().collect());

                res
            },
            RecordData::AAAA(addr) => {
                addr.octets().to_vec()
            },
            RecordData::UNKNOWN(n) => {
                vec![0; *n as usize]
            }
        }
    }
}
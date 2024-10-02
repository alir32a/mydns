use anyhow::Result;
use std::net::{Ipv4Addr, Ipv6Addr};
use crate::bytes_util::BytesUtil;
use crate::query_class::QueryClass;
use crate::parser::PacketParser;
use crate::query_type::QueryType;
use crate::pair::BytesPair;
use crate::writer::PacketWriter;

#[derive(Debug, Clone)]
pub struct Record {
    pub domain: String,
    pub rtype: QueryType,
    pub rclass: QueryClass,
    pub ttl: u32,
    pub len: u16,
    pub data: RecordData,
}

impl Record {
    pub fn parse(parser: &mut PacketParser) -> Result<Record> {
        let domain = parser.parse_domain_name()?;

        let rtype = QueryType::from(parser.next_u16()?);
        let rclass = QueryClass::from(parser.next_u16()?);
        let ttl = parser.next_u32()?;
        let len = parser.next_u16()?;

        let mut record = Record {
            domain: domain.clone(),
            rtype,
            rclass,
            ttl,
            len,
            data: RecordData::UNKNOWN(len),
        };

        match record.rtype {
            QueryType::A => {
                let raw_addr = parser.next_u32()?;
                record.data = RecordData::A(
                    Ipv4Addr::new(
                        ((raw_addr >> 24) & 0xFF) as u8,
                        ((raw_addr >> 16) & 0xFF) as u8,
                        ((raw_addr >> 8) & 0xFF) as u8,
                        ((raw_addr >> 0) & 0xFF) as u8,
                    )
                );

                Ok(record)
            },
            QueryType::NS => {
                record.data = RecordData::NS(parser.parse_domain_name()?);

                Ok(record)
            },
            QueryType::CNAME => {
                record.data = RecordData::CNAME(parser.parse_domain_name()?);

                Ok(record)
            },
            QueryType::PTR => {
                record.data = RecordData::PTR(parser.parse_domain_name()?);

                Ok(record)
            },
            QueryType::TXT => {
                record.data = RecordData::TXT(parser.parse_domain_name()?);

                Ok(record)
            },
            QueryType::SOA => {
                record.data = RecordData::SOA {
                      mname: parser.parse_domain_name()?,
                      rname: parser.parse_domain_name()?,
                      serial: parser.next_u32()?,
                      refresh: parser.next_u32()?,
                      retry: parser.next_u32()?,
                      expire: parser.next_u32()?,
                      minimum: parser.next_u32()?,
                };

                Ok(record)
            },
            QueryType::MX => {
                record.data = RecordData::MX {
                    preference: parser.next_u16()?,
                    exchange: parser.parse_domain_name()?,
                };

                Ok(record)
            },
            QueryType::AAAA => {
                let first_part = parser.next_u32()?;
                let second_part = parser.next_u32()?;
                let third_part = parser.next_u32()?;
                let fourth_part = parser.next_u32()?;

                record.data = RecordData::AAAA(Ipv6Addr::new(
                    ((first_part >> 16) & 0xFFFF) as u16,
                    (first_part & 0xFFFF) as u16,
                    ((second_part >> 16) & 0xFFFF) as u16,
                    (second_part & 0xFFFF) as u16,
                    ((third_part >> 16) & 0xFFFF) as u16,
                    (third_part & 0xFFFF) as u16,
                    ((fourth_part >> 16) & 0xFFFF) as u16,
                    (fourth_part & 0xFFFF) as u16,
                ));

                Ok(record)
            }
            _ => {
                Ok(record)
            }
        }
    }
}

#[derive(Clone, Debug)]
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
    PTR(String),
    HINFO {
        cpu: String,
        os: String
    },
    MX {
        preference: u16,
        exchange: String
    },
    TXT(String),
    AAAA(Ipv6Addr),
    SRV {
        priority: u16,
        weight: u16,
        port: u16,
        host: String,
    },
    UNKNOWN(u16)
}

impl RecordData {
    pub fn bytes(&self) -> Result<Vec<u8>> {
        match self {
            RecordData::A(addr) => {
                Ok(addr.octets().to_vec())
            },
            RecordData::NS(host) | RecordData::CNAME(host) |
            RecordData::PTR(host) | RecordData::TXT(host) => {
                PacketWriter::write_domain(host)
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
                let mut res: Vec<u8> = PacketWriter::write_domain(mname)?;
                res.append(&mut PacketWriter::write_domain(rname)?);
                res.append(&mut BytesUtil::from_u32(*serial).to_vec());
                res.append(&mut BytesUtil::from_u32(*refresh).to_vec());
                res.append(&mut BytesUtil::from_u32(*retry).to_vec());
                res.append(&mut BytesUtil::from_u32(*expire).to_vec());
                res.append(&mut BytesUtil::from_u32(*minimum).to_vec());

                Ok(res)
            },
            RecordData::HINFO { ref cpu, ref os} => {
                let mut res: Vec<u8> = cpu.bytes().collect();
                res.append(&mut os.bytes().collect());

                Ok(res)
            },
            RecordData::MX { preference, exchange } => {
                let mut res: Vec<u8> = BytesPair::from(*preference).bytes();
                res.append(&mut PacketWriter::write_domain(exchange)?);

                Ok(res)
            },
            RecordData::AAAA(addr) => {
                Ok(addr.octets().to_vec())
            },
            RecordData::SRV {
                priority,
                weight,
                port,
                host
            } => {
                let mut res = BytesPair::from(*priority).bytes();
                res.append(&mut BytesPair::from(*weight).bytes());
                res.append(&mut BytesPair::from(*port).bytes());
                res.append(&mut PacketWriter::write_domain(host)?);

                Ok(res)
            },
            RecordData::UNKNOWN(n) => {
                Ok(vec![0; *n as usize])
            }
        }
    }
}
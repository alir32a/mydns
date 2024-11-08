use anyhow::Result;
use std::net::{Ipv4Addr, Ipv6Addr};
use crate::query_class::QueryClass;
use crate::parser::PacketParser;
use crate::query_type::QueryType;

#[derive(Default, Debug, Clone)]
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

impl Default for RecordData {
    fn default() -> Self {
        Self::UNKNOWN(u16::default())
    }
}
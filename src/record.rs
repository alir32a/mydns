use anyhow::Result;
use std::net::Ipv4Addr;
use crate::parser::PacketParser;
use crate::query_type::QueryType;

#[derive(Debug)]
pub enum Record {
    A {
        domain: String,
        addr: Ipv4Addr,
        ttl: u32,
    },
    UNKNOWN {
        domain: String,
        qtype: u16,
        qclass: u16,
        ttl: u32,
        len: u16,
    }
}

impl Record {
    pub fn parse(parser: &mut PacketParser, domain: &String) -> Result<Record> {
        // reading name which we are getting from the argument, so we don't need it.
        let _ = parser.next_u16()?;

        let qtype = QueryType::from_num(parser.next_u16()?);
        let qclass = parser.next_u16()?;
        let ttl = parser.next_u32()?;
        let len = parser.next_u16()?;

        match qtype {
            QueryType::A => {
                let raw_addr = parser.next_u32()?;
                let addr = Ipv4Addr::new(
                    ((raw_addr >> 24) & 0xFF) as u8,
                    ((raw_addr >> 16) & 0xFF) as u8,
                    ((raw_addr >> 8) & 0xFF) as u8,
                    ((raw_addr >> 0) & 0xFF) as u8,
                );

                Ok(Record::A {
                    domain: domain.clone(),
                    addr,
                    ttl,
                })
            },
            QueryType::UNKNOWN(_) => {
                Ok(Record::UNKNOWN {
                    domain: domain.clone(),
                    qtype: qtype.to_num(),
                    qclass,
                    ttl,
                    len,
                })
            }
        }
    }
}
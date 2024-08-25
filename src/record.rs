use anyhow::Result;
use std::net::Ipv4Addr;
use crate::dns_class::DNSClass;
use crate::parser::PacketParser;
use crate::dns_type::DNSType;
use crate::record_data::RecordData;

#[derive(Debug)]
pub struct Record {
    pub domain: String,
    pub rtype: DNSType,
    pub rclass: DNSClass,
    pub ttl: u32,
    pub len: u16,
    pub data: RecordData,
}

impl Record {
    pub fn parse(parser: &mut PacketParser, domain: &String) -> Result<Record> {
        // reading name which we are getting from the argument, so we don't need it.
        let _ = parser.next_u16()?;

        let rtype = DNSType::from(parser.next_u16()?);
        let rclass = DNSClass::from(parser.next_u16()?);
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
            DNSType::A => {
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
            _ => {
                Ok(record)
            }
        }
    }
}
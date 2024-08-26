use anyhow::Result;
use std::net::{Ipv4Addr, Ipv6Addr};
use crate::dns_class::DNSClass;
use crate::parser::PacketParser;
use crate::dns_type::DNSType;
use crate::pair::BytesPair;
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
    pub fn parse(parser: &mut PacketParser) -> Result<Record> {
        let domain = parser.parse_domain_name()?;

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
            DNSType::NS | DNSType::CNAME => {
                record.data = RecordData::NS(parser.parse_domain_name()?);

                Ok(record)
            },
            DNSType::MX => {
                record.data = RecordData::MX {
                    preference: parser.next_u16()?,
                    exchange: parser.parse_domain_name()?,
                };

                Ok(record)
            },
            DNSType::AAAA => {
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
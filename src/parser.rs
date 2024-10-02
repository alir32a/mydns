use anyhow::{bail, Result};
use crate::query_class::QueryClass;
use crate::header::Header;
use crate::packet::Packet;
use crate::pair::BytesPair;
use crate::query_type::QueryType;
use crate::question::Question;
use crate::record::Record;

pub struct PacketParser {
    buf: Vec<u8>,
    offset: u16
}

impl PacketParser {
    pub fn new(data: &[u8]) -> PacketParser {
        PacketParser {
            buf: data.to_vec(),
            offset: 0,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        self.buf.as_slice()
    }

    pub fn offset(&self) -> u16 {
        self.offset
    }

    pub fn seek(&mut self, n: u16) -> Result<()> {
        self.offset = n;

        Ok(())
    }

    pub fn next(&mut self) -> Result<u8> {
        if self.offset >= 512 {
            bail!("End Of Buffer");
        }

        let res = self.buf[self.offset as usize];
        self.offset += 1;

        Ok(res)
    }

    pub fn next_u16(&mut self) -> Result<u16> {
        let res = ((self.next()? as u16) << 8) | self.next()? as u16;

        Ok(res)
    }

    pub fn next_u32(&mut self) -> Result<u32> {
        let res = ((self.next()? as u32) << 24)
            | ((self.next()? as u32) << 16)
            | ((self.next()? as u32) << 8)
            | self.next()? as u32;

        Ok(res)
    }

    pub fn get(&self, n: u16) -> Result<u8> {
        if n >= 512 {
            bail!("End Of Buffer");
        }

        let res = self.buf[n as usize];

        Ok(res)
    }

    pub fn range(&self, start: u16, len: u16) -> Result<&[u8]> {
        if start + len >= 512 {
            bail!("End Of Buffer");
        }

        let res = &self.buf[start as usize..(start + len) as usize];

        Ok(&res)
    }

    pub fn parse(&mut self) -> Result<Packet> {
        let mut packet = Packet::new();

        packet.header = self.parse_header()?;

        for _ in 0..packet.header.question_count {
            packet.questions.push(self.parse_question()?);
        }

        for _ in 0..packet.header.answer_count {
            packet.answers.push(Record::parse(self)?);
        }

        for _ in 0..packet.header.authority_count {
            packet.authorities.push(Record::parse(self)?);
        }

        for _ in 0..packet.header.resource_count {
            packet.resources.push(Record::parse(self)?);
        }

        Ok(packet)
    }

    pub fn parse_header(&mut self) -> Result<Header> {
        let mut header = Header::new();

        // seek to the beginning of the packet to parse the header.
        if self.offset != 0 {
            self.seek(0)?;
        }

        header.id = self.next_u16()?;
        self.parse_header_flags(&mut header)?;
        header.question_count = self.next_u16()?;
        header.answer_count = self.next_u16()?;
        header.authority_count = self.next_u16()?;
        header.resource_count = self.next_u16()?;

        Ok(header)
    }

    pub fn parse_header_flags(&mut self, header: &mut Header) -> Result<()> {
        let pair = BytesPair::from(self.next_u16()?);

        header.response = (pair.0 & (1 << 7)) == 1;
        header.opcode = (pair.0 >> 3) & 0x0F;
        header.authoritive = (pair.0 & (1 << 2)) == 1;
        header.truncation = (pair.0 & (1 << 1)) == 1;
        header.recursion_desired = (pair.0 & (1 << 0)) == 1;

        header.recursion_available = (pair.1 & (1 << 7)) == 1;
        header.reserved = (pair.1 >> 4) & 0x07;
        header.code = pair.1 & 0x0F;

        Ok(())
    }

    pub fn parse_question(&mut self) -> Result<Question> {
        let name = self.parse_domain_name()?;
        let qtype = self.next_u16()?;
        let qclass = self.next_u16()?;

        Ok(Question::new_with_class(name, QueryType::from(qtype), QueryClass::from(qclass)))
    }

    pub fn parse_domain_name(&mut self) -> Result<String> {
        let mut res = String::new();

        let mut pos = self.offset();

        let mut total_jumps = 0;
        let max_jumps = 5;

        loop {
            if total_jumps > max_jumps {
                bail!("Max jumps reached while parsing a domain");
            }

            let len = self.get(pos)?;

            if (len & 0xC0) == 0xC0 {
                if total_jumps == 0 {
                    self.seek(pos + 2)?;
                }

                let next_byte = self.get(pos + 1)? as u16;
                let offset = (((len as u16) ^ 0xC0) << 8) | next_byte;
                pos = offset;

                total_jumps += 1;

                continue;
            } else {
                pos += 1;

                if len == 0 {
                    break;
                }

                if !res.is_empty() {
                    res.push('.');
                }

                let bytes = self.range(pos, len as u16)?;
                res.push_str(&String::from_utf8_lossy(bytes).to_lowercase());

                pos += len as u16;
            }
        }

        if total_jumps == 0 {
            self.seek(pos)?;
        }

        Ok(res)
    }
}
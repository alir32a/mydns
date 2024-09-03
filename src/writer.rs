use anyhow::{bail, Result};
use crate::bytes_util::BytesUtil;
use crate::header::Header;
use crate::packet::Packet;
use crate::pair::BytesPair;
use crate::record::Record;

#[derive(Default)]
pub struct PacketWriter {
    pub packet: Packet,
    buf: Vec<u8>,
    offset: u16,
}

impl PacketWriter {
    pub fn new() -> PacketWriter {
        PacketWriter {
            buf: vec![0],
            offset: 0,
            ..Default::default()
        }
    }

    pub fn from(packet: Packet) -> PacketWriter {
        PacketWriter {
            packet,
            buf: Vec::from([0u8; 512]),
            offset: 0,
        }
    }

    pub fn write(&mut self) -> Result<Vec<u8>> {
        // reset the buffer before any writes to the buf
        self.buf.clear();

        self.write_header()?;
        self.write_questions()?;
        self.write_bytes(Self::write_records(&self.packet.answers)?)?;
        self.write_bytes(Self::write_records(&self.packet.authorities)?)?;
        self.write_bytes(Self::write_records(&self.packet.resources)?)?;

        Ok(self.buf.clone())
    }

    fn write_header(&mut self) -> Result<()> {
        self.seek(0)?;

        self.write_u16(self.packet.header.id)?;

        let flags = Self::write_header_flags(&self.packet.header);
        self.write_byte(flags.0)?;
        self.write_byte(flags.1)?;

        self.write_u16(self.packet.header.question_count)?;
        self.write_u16(self.packet.header.answer_count)?;
        self.write_u16(self.packet.header.authority_count)?;
        self.write_u16(self.packet.header.resource_count)?;

        Ok(())
    }

    fn write_questions(&mut self) -> Result<()> {
        if self.offset > 12 {
            self.seek(12)?;
        }

        let mut buf: Vec<u8> = Vec::new();
        for question in &self.packet.questions {
            let domain = Self::write_domain(&question.domain)?;
            for byte in domain {
                buf.push(byte);
            }

            let pair = BytesPair::from(question.qtype.to_num());
            buf.append(&mut pair.bytes());

            let pair = BytesPair::from(question.qclass.to_num());
            buf.append(&mut pair.bytes());
        }

        self.write_bytes(buf)?;

        Ok(())
    }

    fn write_records(records: &Vec<Record>) -> Result<Vec<u8>> {
        let mut res = Vec::new();

        for record in records {
            let mut bytes = Self::write_domain(&record.domain)?;
            res.append(&mut bytes);

            res.append(&mut BytesPair::from(record.rtype.to_num()).bytes());
            res.append(&mut BytesPair::from(record.rclass.to_num()).bytes());

            res.append(&mut BytesUtil::from_u32(record.ttl).to_vec());

            let mut data = record.data.bytes()?;
            res.append(&mut BytesPair::from(data.len() as u16).bytes());
            res.append(&mut data);
        }

        Ok(res)
    }

    pub fn write_domain(domain: &String) -> Result<Vec<u8>> {
        let mut res = Vec::new();

        for label in domain.split('.') {
            if label.len() > 63 {
                bail!("labels exceeds 63 character limit");
            }

            res.push(label.len() as u8);
            for byte in label.as_bytes() {
                res.push(*byte);
            }
        }

        res.push(0x00);

        Ok(res)
    }

    fn write_header_flags(header: &Header) -> (u8, u8) {
        let mut res = (0u8, 0u8);

        res.0 = header.recursion_desired as u8
            | (header.truncation as u8) << 1
            | (header.authoritive as u8) << 2
            | ((header.opcode & 0x0F) << 3)
            | (header.response as u8) << 7;

        res.1 = header.code & 0x0F
            | ((header.reserved & 0x07) << 4)
            | (header.recursion_available as u8) << 7;

        res
    }

    fn seek(&mut self, n: u16) -> Result<()> {
        if n >= 512 {
            bail!("End Of Stream");
        }

        self.offset = n;

        Ok(())
    }

    fn write_byte(&mut self, value: u8) -> Result<()> {
        if self.offset >= 512 {
            bail!("End Of Buffer");
        }

        self.buf.insert(self.offset as usize, value);
        self.offset += 1;

        Ok(())
    }

    fn write_bytes(&mut self, mut values: Vec<u8>) -> Result<()> {
        if self.offset + values.len() as u16 > 512 {
            bail!("End Of Buffer");
        }

        self.offset += values.len() as u16;
        self.buf.append(values.as_mut());

        Ok(())
    }

    fn write_u16(&mut self, value: u16) -> Result<()> {
        self.write_byte(((value >> 8) & 0xFF) as u8)?;
        self.write_byte((value & 0xFF) as u8)?;

        Ok(())
    }

    fn write_u32(&mut self, value: u32) -> Result<()> {
        self.write_byte(((value >> 24) & 0xFF) as u8)?;
        self.write_byte(((value >> 16) & 0xFF) as u8)?;
        self.write_byte(((value >> 8) & 0xFF) as u8)?;
        self.write_byte((value & 0xFF) as u8)?;

        Ok(())
    }
}
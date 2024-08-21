use anyhow::{bail, Result};
use crate::header::Header;
use crate::question::Question;
use crate::util::{bool_to_u8, u16_to_u8};

#[derive(Default, Debug)]
pub struct PacketWriter {
    buf: Vec<u8>,
    offset: u16,
    questions_len: u16,
    answers_offset: u16,
    answers_len: u16,
    authorities_offset: u16,
    authorities_len: u16,
    resources_offset: u16,
}

impl PacketWriter {
    pub fn new() -> PacketWriter {
        PacketWriter {
            buf: Vec::with_capacity(512),
            offset: 0,
            ..Default::default()
        }
    }

    pub fn write_header(&mut self, header: &Header) -> Result<()> {
        self.seek(0)?;

        self.write_u16(header.id)?;
        self.write_header_flags(header)?;
        self.write_u16(header.question_count)?;
        self.write_u16(header.answer_count)?;
        self.write_u16(header.authority_count)?;
        self.write_u16(header.answer_count)?;

        Ok(())
    }

    pub fn write_question(&mut self, question: &Question) -> Result<()> {
        let mut pos = 12 + self.questions_len as usize;
        let mut len = 0;

        for byte in Self::write_domain(&question.domain)? {
            self.buf.insert(pos, byte);
            pos += 1;
            len += 1;
        }

        let (first, second) = u16_to_u8(question.qtype.to_num());
        self.buf.insert(pos, first);
        self.buf.insert(pos + 1, second);
        pos += 2;
        len += 2;

        let (first, second) = u16_to_u8(question.class);
        self.buf.insert(pos, first);
        self.buf.insert(pos + 1, second);
        pos += 2;
        len += 2;

        self.questions_len = len;

        Ok(())
    }

    fn write_domain(domain: &String) -> Result<Vec<u8>> {
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

    fn write_header_flags(&mut self, header: &Header) -> Result<()> {
        let (mut first, mut second) = (0u8, 0u8);

        first = bool_to_u8(header.recursion_desired)
            | (bool_to_u8(header.truncation) << 1)
            | (bool_to_u8(header.authoritive) << 2)
            | ((header.opcode & 0xF) << 3)
            | (bool_to_u8(header.response) << 7);

        second = header.code & 0xF
            | ((header.reserved & 0x7) << 4)
            | bool_to_u8(header.recursion_available) << 7;

        self.write(first)?;
        self.write(second)?;

        Ok(())
    }

    fn seek(&mut self, n: u16) -> Result<()> {
        if n >= 512 {
            bail!("End Of Stream");
        }

        self.offset = n;

        Ok(())
    }

    fn write(&mut self, value: u8) -> Result<()> {
        if self.offset >= 512 {
            bail!("End Of Buffer");
        }

        self.buf[self.offset as usize] = value;
        self.offset += 1;

        Ok(())
    }

    fn write_u16(&mut self, value: u16) -> Result<()> {
        self.write(((value >> 8) & 0xFF) as u8)?;
        self.write((value & 0xFF) as u8)?;

        Ok(())
    }

    fn write_u32(&mut self, value: u32) -> Result<()> {
        self.write(((value >> 24) & 0xFF) as u8)?;
        self.write(((value >> 16) & 0xFF) as u8)?;
        self.write(((value >> 8) & 0xFF) as u8)?;
        self.write((value & 0xFF) as u8)?;

        Ok(())
    }
}
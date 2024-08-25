use anyhow::{bail, Result};
use std::io::{Write, Read};

#[derive(Default, Debug)]
pub struct BytesPair(pub(crate) u8, pub(crate) u8);

impl BytesPair {
    pub fn new(first: u8, second: u8) -> Self {
        Self(first, second)
    }

    pub fn from(value: u16) -> Self {
        Self((value >> 8) as u8, (value & 0xFF) as u8)
    }

    pub fn to_u16(&self) -> u16 {
        (self.0 as u16) << 8 | self.1 as u16
    }

    pub fn bytes(&self) -> Vec<u8> {
        [self.0, self.1].to_vec()
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 2];

        let n = reader.read(&mut buf)?;
        if n < 2 {
            bail!("Read from buffer failed");
        }

        Ok(Self(buf[0], buf[1]))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<usize> {
        let n = writer.write(&[self.0, self.1])?;
        if n < 2 {
            bail!("Write to the buffer failed");
        }

        Ok(n)
    }
}
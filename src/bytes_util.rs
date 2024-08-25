use anyhow::{bail, Result};

pub struct BytesUtil;

impl BytesUtil {
    pub fn from_u32(value: u32) -> [u8; 4] {
        [
            ((value >> 24) & 0xFF) as u8,
            ((value >> 16) & 0xFF) as u8,
            ((value >> 8) & 0xFF) as u8,
            (value & 0xFF) as u8
        ]
    }

    pub fn parse_u32(bytes: &[u8]) -> Result<u32> {
        if bytes.len() != 4 {
            bail!("parse_u32: Wrong number of bytes to parse");
        }

        Ok(
            (bytes[0] as u32) << 24
            | (bytes[1] as u32) << 16
            | (bytes[2] as u32) << 8
            | bytes[3] as u32
        )
    }
}
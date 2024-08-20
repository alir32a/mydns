use std::io::{Result};
use crate::util::{u16_to_u8,bool_to_u8};

#[derive(Default, Debug)]
pub struct Header {
    pub id: u16,
    pub response: bool,
    pub opcode: u8,
    pub authoritive: bool,
    pub truncation: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,
    pub reserved: u8,
    pub code: u8,
    pub question_count: u16,
    pub answer_count: u16,
    pub authority_count: u16,
    pub additional_count: u16
}

impl Header {
    pub fn new() -> Header {
        Header::default()
    }

    pub fn new_with_id(id: u16) -> Header {
        Header {
            id,
            ..Default::default()
        }
    }

    pub fn with_query_res_indicator(mut self) -> Self {
        self.response = true;

        self
    }

    pub fn with_question_count(mut self, n: u16) -> Self {
        self.question_count = n;

        self
    }

    pub fn write(&self) -> [u8; 12] {
        let mut res: [u8; 12] = Default::default();

        (res[0], res[1]) = u16_to_u8(self.id);
        (res[2], res[3]) = self.write_flags();
        (res[4], res[5]) = u16_to_u8(self.question_count);
        (res[6], res[7]) = u16_to_u8(self.answer_count);
        (res[8], res[9]) = u16_to_u8(self.authority_count);
        (res[10], res[11]) = u16_to_u8(self.additional_count);

        res
    }

    fn write_flags(&self) -> (u8, u8) {
        let (mut first, mut second) = (0u8, 0u8);

        first = bool_to_u8(self.recursion_desired)
            | (bool_to_u8(self.truncation) << 1)
            | (bool_to_u8(self.authoritive) << 2)
            | ((self.opcode & 0xF) << 3)
            | (bool_to_u8(self.response) << 7);

        second = self.code & 0xF
            | ((self.reserved & 0xF) << 4)
            | bool_to_u8(self.recursion_available) << 7;

        (first, second)
    }
}
use crate::query_type::QueryType;
use crate::util::{u16_to_u8};

pub struct Question {
    pub domain: String,
    pub qtype: QueryType,
    pub class: u16
}

impl Question {
    pub fn new(name: String, qtype: QueryType, qclass: u16) -> Question {
        Question {
            domain: name,
            qtype,
            class: qclass,
        }
    }

    pub fn write(&self) -> Vec<u8> {
        let mut res: Vec<u8> = vec![];

        let splitted_name = self.domain.split(".");

        for value in splitted_name.into_iter() {
            res.push(value.len() as u8);

            res.append(&mut value.as_bytes().to_vec());
        }
        res.push(0x0);

        let qtype = u16_to_u8(self.qtype.to_num());
        res.append(&mut vec![qtype.0, qtype.1]);

        let class = u16_to_u8(self.class);
        res.append(&mut vec![class.0, class.1]);

        res
    }
}
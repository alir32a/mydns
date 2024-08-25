use crate::dns_class::DNSClass;
use crate::pair::BytesPair;
use crate::dns_type::DNSType;

pub struct Question {
    pub domain: String,
    pub qtype: DNSType,
    pub qclass: DNSClass
}

impl Question {
    pub fn new(name: String, qtype: DNSType, qclass: DNSClass) -> Question {
        Question {
            domain: name,
            qtype,
            qclass,
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

        let qtype = BytesPair::from(self.qtype.to_num());
        res.append(&mut qtype.bytes());

        let class = BytesPair::from(self.qclass.to_num());
        res.append(&mut class.bytes());

        res
    }
}
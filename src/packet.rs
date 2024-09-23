use rand::random;
use crate::dns_type::DNSType;
use crate::header::Header;
use crate::question::Question;
use crate::record::Record;

#[derive(Default)]
pub struct Packet {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<Record>,
    pub authorities: Vec<Record>,
    pub resources: Vec<Record>,
}

impl Packet {
    pub fn new() -> Packet {
        Packet {
            ..Default::default()
        }
    }

    pub fn from(packet: &Packet) -> Packet {
        Packet {
            header: packet.header.clone(),
            questions: packet.questions.clone(),
            ..Default::default()
        }
    }

    pub fn get_empty_packet() -> Packet {
        Self {
            header: Header {
                id: random(),
                recursion_desired: true,
                question_count: 1,
                ..Default::default()
            },
            questions: vec![Question::new(".".to_string(), DNSType::A)],
            ..Default::default()
        }
    }
}
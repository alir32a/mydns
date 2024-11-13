#[derive(Clone, Default, Debug)]
pub struct Header {
    pub id: u16,
    pub response: bool,
    pub opcode: u8,
    pub authoritative: bool,
    pub truncation: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,
    pub reserved: u8,
    pub code: u8,
    pub question_count: u16,
    pub answer_count: u16,
    pub authority_count: u16,
    pub resource_count: u16
}

impl Header {
    pub fn new() -> Header {
        Header::default()
    }
}
use std::net::Ipv4Addr;

#[derive(Debug)]
pub enum RecordData {
    A(Ipv4Addr),
    ASTERISK(Box<[u8]>),
    UNKNOWN(u16)
}

impl RecordData {
    pub fn bytes(&self) -> Vec<u8> {
        match self {
            RecordData::A(addr) => {
                addr.octets().to_vec()
            }
            RecordData::ASTERISK(data) => {
                data.to_vec()
            }
            RecordData::UNKNOWN(n) => {
                vec![0; *n as usize]
            }
        }
    }
}
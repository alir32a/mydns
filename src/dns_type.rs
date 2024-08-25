#[derive(Default, PartialEq, Eq, Debug, Clone, Hash, Copy)]
pub enum DNSType {
    #[default]
    A, // 1
    NS,
    MD,
    MF,
    CNAME,
    SOA,
    MB,
    MG,
    MR,
    NULL,
    WKS,
    PTR,
    HINFO,
    MINFO,
    MX,
    TXT,
    AAAA, // 28
    // QTYPE
    AXFR, // 252
    MAILB,
    MAILA,
    ASTERISK,
}

impl DNSType {
    pub fn from(value: u16) -> DNSType {
        match value {
            1 => DNSType::A,
            2 => DNSType::NS,
            3 => DNSType::MD,
            4 => DNSType::MF,
            5 => DNSType::CNAME,
            6 => DNSType::SOA,
            7 => DNSType::MB,
            8 => DNSType::MG,
            9 => DNSType::MR,
            10 => DNSType::NULL,
            11 => DNSType::WKS,
            12 => DNSType::PTR,
            13 => DNSType::HINFO,
            14 => DNSType::MINFO,
            15 => DNSType::MX,
            16 => DNSType::TXT,
            28 => DNSType::AAAA,
            252 => DNSType::AXFR,
            253 => DNSType::MAILB,
            254 => DNSType::MAILA,
            _ => DNSType::ASTERISK,
        }
    }

    pub fn to_num(&self) -> u16 {
        match *self {
            DNSType::A => 1,
            DNSType::NS => 2,
            DNSType::MD => 3,
            DNSType::MF => 4,
            DNSType::CNAME => 5,
            DNSType::SOA => 6,
            DNSType::MB => 7,
            DNSType::MG => 8,
            DNSType::MR => 9,
            DNSType::NULL => 10,
            DNSType::WKS => 11,
            DNSType::PTR => 12,
            DNSType::HINFO => 13,
            DNSType::MINFO => 14,
            DNSType::MX => 15,
            DNSType::TXT => 16,
            DNSType::AAAA => 28,
            DNSType::AXFR => 252,
            DNSType::MAILB => 253,
            DNSType::MAILA => 254,
            DNSType::ASTERISK => 255,
        }
    }
}
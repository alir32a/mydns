#[derive(Default, PartialEq, Eq, Debug, Clone, Hash, Copy)]
pub enum DNSType {
    #[default]
    A, // 1
    NS,
    CNAME,
    SOA,
    PTR,
    HINFO,
    MX,
    TXT,
    AAAA,
    SRV, // 33
    OPT, // 41
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
            5 => DNSType::CNAME,
            6 => DNSType::SOA,
            12 => DNSType::PTR,
            13 => DNSType::HINFO,
            15 => DNSType::MX,
            16 => DNSType::TXT,
            28 => DNSType::AAAA,
            33 => DNSType::SRV,
            41 => DNSType::OPT,
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
            DNSType::CNAME => 5,
            DNSType::SOA => 6,
            DNSType::PTR => 12,
            DNSType::HINFO => 13,
            DNSType::MX => 15,
            DNSType::TXT => 16,
            DNSType::AAAA => 28,
            DNSType::SRV => 33,
            DNSType::OPT => 41,
            DNSType::AXFR => 252,
            DNSType::MAILB => 253,
            DNSType::MAILA => 254,
            DNSType::ASTERISK => 255,
        }
    }
}
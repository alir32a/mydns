#[derive(Default, PartialEq, Eq, Debug, Clone, Hash, Copy)]
pub enum QueryType {
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

impl QueryType {
    pub fn from(value: u16) -> QueryType {
        match value {
            1 => QueryType::A,
            2 => QueryType::NS,
            5 => QueryType::CNAME,
            6 => QueryType::SOA,
            12 => QueryType::PTR,
            13 => QueryType::HINFO,
            15 => QueryType::MX,
            16 => QueryType::TXT,
            28 => QueryType::AAAA,
            33 => QueryType::SRV,
            41 => QueryType::OPT,
            252 => QueryType::AXFR,
            253 => QueryType::MAILB,
            254 => QueryType::MAILA,
            _ => QueryType::ASTERISK,
        }
    }

    pub fn to_num(&self) -> u16 {
        match *self {
            QueryType::A => 1,
            QueryType::NS => 2,
            QueryType::CNAME => 5,
            QueryType::SOA => 6,
            QueryType::PTR => 12,
            QueryType::HINFO => 13,
            QueryType::MX => 15,
            QueryType::TXT => 16,
            QueryType::AAAA => 28,
            QueryType::SRV => 33,
            QueryType::OPT => 41,
            QueryType::AXFR => 252,
            QueryType::MAILB => 253,
            QueryType::MAILA => 254,
            QueryType::ASTERISK => 255,
        }
    }
}
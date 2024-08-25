#[derive(Default, Debug)]
pub enum DNSClass {
    #[default]
    IN, // 1
    CS,
    CH,
    HS,
    ASTERISK
}

impl DNSClass {
    pub fn from(value: u16) -> Self {
        match value {
            1 => DNSClass::IN,
            2 => DNSClass::CS,
            3 => DNSClass::CH,
            4 => DNSClass::HS,
            _ => DNSClass::ASTERISK
        }
    }

    pub fn to_num(&self) -> u16 {
        match *self {
            DNSClass::IN => 1,
            DNSClass::CS => 2,
            DNSClass::CH => 3,
            DNSClass::HS => 4,
            DNSClass::ASTERISK => 255
        }
    }
}
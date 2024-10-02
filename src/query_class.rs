#[derive(Clone, Default, Debug)]
pub enum QueryClass {
    #[default]
    IN, // 1
    CS,
    CH,
    HS,
    ASTERISK
}

impl QueryClass {
    pub fn from(value: u16) -> Self {
        match value {
            1 => QueryClass::IN,
            2 => QueryClass::CS,
            3 => QueryClass::CH,
            4 => QueryClass::HS,
            _ => QueryClass::ASTERISK
        }
    }

    pub fn to_num(&self) -> u16 {
        match *self {
            QueryClass::IN => 1,
            QueryClass::CS => 2,
            QueryClass::CH => 3,
            QueryClass::HS => 4,
            QueryClass::ASTERISK => 255
        }
    }
}
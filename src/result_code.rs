#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ResultCode {
    NOERROR = 0,
    FORMERR = 1,
    SERVFAIL = 2,
    NXDOMAIN = 3,
    NOTIMP = 4,
    REFUSED = 5,
    YXDOMAIN = 6,
    XRRSET = 7,
    NOTAUTH = 8,
    NOTZONE = 9,
}

impl ResultCode {
    pub fn from(value: u8) -> ResultCode {
        match value {
            1 => ResultCode::FORMERR,
            2 => ResultCode::SERVFAIL,
            3 => ResultCode::NXDOMAIN,
            4 => ResultCode::NOTIMP,
            5 => ResultCode::REFUSED,
            6 => ResultCode::YXDOMAIN,
            7 => ResultCode::XRRSET,
            8 => ResultCode::NOTAUTH,
            9 => ResultCode::NOTZONE,
            0 | _ => ResultCode::NOERROR,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            ResultCode::NOERROR => 0,
            ResultCode::FORMERR => 1,
            ResultCode::SERVFAIL => 2,
            ResultCode::NXDOMAIN => 3,
            ResultCode::NOTIMP => 4,
            ResultCode::REFUSED => 5,
            ResultCode::YXDOMAIN => 6,
            ResultCode::XRRSET => 7,
            ResultCode::NOTAUTH => 8,
            ResultCode::NOTZONE => 9,
        }
    }
}
pub fn u16_to_u8(value: u16) -> (u8, u8) {
    let first = (value >> 8) as u8;
    let second = (value & 0xFF) as u8;

    (first, second)
}

pub fn bool_to_u8(value: bool) -> u8 {
    if value {
        return 1;
    }

    0
}
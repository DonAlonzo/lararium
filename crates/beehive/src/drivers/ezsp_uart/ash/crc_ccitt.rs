pub fn crc_ccitt(msg: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for byte in msg.iter() {
        let mut x = ((crc >> 8) ^ (*byte as u16)) & 255;
        x ^= x >> 4;
        crc = (x << 12) ^ (crc << 8) ^ (x << 5) ^ x;
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc_ccitt_1() {
        let crc = crc_ccitt(&[0x53, 0x42, 0xA1, 0xA8, 0x56, 0x28, 0x04, 0x82]);
        assert_eq!(0x032A, crc);
    }

    #[test]
    fn test_crc_ccitt_2() {
        let crc = crc_ccitt(&[0x25, 0x42, 0x21, 0xA8, 0x56]);
        assert_eq!(0xA609, crc);
    }
}

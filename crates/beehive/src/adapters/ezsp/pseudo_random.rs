use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::sync::watch;

pub struct PseudoRandom {
    value: u8,
}

impl PseudoRandom {
    pub fn new() -> Self {
        PseudoRandom { value: 0x42 }
    }
}

impl Iterator for PseudoRandom {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let current_value = self.value;
        if self.value & 1 == 0 {
            self.value >>= 1;
        } else {
            self.value = (self.value >> 1) ^ 0xB8;
        }
        Some(current_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pseudo_random() {
        let mut random_sequence = PseudoRandom::new();
        assert_eq!(0x42, random_sequence.next().unwrap());
        assert_eq!(0x21, random_sequence.next().unwrap());
        assert_eq!(0xA8, random_sequence.next().unwrap());
        assert_eq!(0x54, random_sequence.next().unwrap());
        assert_eq!(0x2A, random_sequence.next().unwrap());
    }
}

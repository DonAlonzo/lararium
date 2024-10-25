use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberEUI64([u8; 8]);

impl EmberEUI64 {
    pub fn new(value: [u8; 8]) -> Self {
        Self(value)
    }

    pub fn new_blank() -> Self {
        Self([0xFF; 8])
    }
}

impl std::fmt::Display for EmberEUI64 {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6], self.0[7]
        )
    }
}

impl Decode for EmberEUI64 {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 8 {
            return Err(DecodeError::InsufficientData);
        }
        let mut value = [0; 8];
        buffer.copy_to_slice(&mut value);
        Ok(Self(value))
    }
}

impl Encode for EmberEUI64 {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_slice(&self.0);
    }
}

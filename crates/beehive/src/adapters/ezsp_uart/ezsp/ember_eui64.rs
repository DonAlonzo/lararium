use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberEUI64([u8; 8]);

impl EmberEUI64 {
    pub fn new(value: [u8; 8]) -> Self {
        Self(value)
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

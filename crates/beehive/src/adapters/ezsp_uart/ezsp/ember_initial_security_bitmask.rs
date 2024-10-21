use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberInitialSecurityBitmask(u16);

impl EmberInitialSecurityBitmask {
    pub fn new() -> Self {
        Self(0)
    }
}

impl Decode for EmberInitialSecurityBitmask {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 2 {
            return Err(DecodeError::InsufficientData);
        }
        Ok(Self(buffer.get_u16_le()))
    }
}

impl Encode for EmberInitialSecurityBitmask {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u16_le(self.0);
    }
}

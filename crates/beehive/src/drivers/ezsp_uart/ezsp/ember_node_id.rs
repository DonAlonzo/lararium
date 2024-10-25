use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberNodeId(u16);

impl EmberNodeId {
    pub fn new(value: u16) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for EmberNodeId {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{:04X}", self.0)
    }
}

impl Decode for EmberNodeId {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 2 {
            return Err(DecodeError::InsufficientData);
        }
        Ok(Self(buffer.get_u16_le()))
    }
}

impl Encode for EmberNodeId {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u16_le(self.0);
    }
}

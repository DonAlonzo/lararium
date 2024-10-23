use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Empty;

impl Decode for Empty {
    fn try_decode_from<B: Buf>(_buffer: &mut B) -> Result<Self, DecodeError> {
        Ok(Self)
    }
}

impl Encode for Empty {
    fn encode_to<B: BufMut>(
        &self,
        _buffer: &mut B,
    ) {
    }
}

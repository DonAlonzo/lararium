use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionResponse {
    pub protocol_version: u8,
    pub stack_type: u8,
    pub stack_version: u16,
}

impl Decode for VersionResponse {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 4 {
            return Err(DecodeError::InsufficientData);
        }
        Ok(Self {
            protocol_version: buffer.get_u8(),
            stack_type: buffer.get_u8(),
            stack_version: buffer.get_u16_le(),
        })
    }
}

impl Encode for VersionResponse {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u8(self.protocol_version);
        buffer.put_u8(self.stack_type);
        buffer.put_u16_le(self.stack_version);
    }
}

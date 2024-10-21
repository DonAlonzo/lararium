use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberVersionCommand {
    pub desired_protocol_version: u8,
}

impl Decode for EmberVersionCommand {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        Ok(Self {
            desired_protocol_version: buffer.get_u8(),
        })
    }
}

impl Encode for EmberVersionCommand {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u8(self.desired_protocol_version);
    }
}

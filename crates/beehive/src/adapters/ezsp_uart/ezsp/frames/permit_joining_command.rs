use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermitJoiningCommand {
    pub duration: u8,
}

impl Decode for PermitJoiningCommand {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        Ok(Self {
            duration: buffer.get_u8(),
        })
    }
}

impl Encode for PermitJoiningCommand {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u8(self.duration);
    }
}

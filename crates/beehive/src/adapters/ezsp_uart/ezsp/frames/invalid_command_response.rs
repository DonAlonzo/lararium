use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidCommandResponse {
    pub status: EzspStatus,
}

impl Decode for InvalidCommandResponse {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        Ok(Self {
            status: EzspStatus::try_decode_from(buffer)?,
        })
    }
}

impl Encode for InvalidCommandResponse {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.status.encode_to(buffer);
    }
}

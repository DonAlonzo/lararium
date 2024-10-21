use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownCommandResponse {
    pub status: EzspStatus,
}

impl Decode for UnknownCommandResponse {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        Some(Self {
            status: EzspStatus::try_decode_from(buffer)?,
        })
    }
}

impl Encode for UnknownCommandResponse {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.status.encode_to(buffer);
    }
}

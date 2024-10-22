use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetInitialSecurityStateResponse {
    pub status: EmberStatus,
}

impl Decode for SetInitialSecurityStateResponse {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        Ok(Self {
            status: EmberStatus::try_decode_from(buffer)?,
        })
    }
}

impl Encode for SetInitialSecurityStateResponse {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.status.encode_to(buffer);
    }
}

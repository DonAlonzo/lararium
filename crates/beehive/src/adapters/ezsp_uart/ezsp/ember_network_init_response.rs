use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberNetworkInitResponse {
    pub status: EmberStatus,
}

impl Decode for EmberNetworkInitResponse {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        Ok(Self {
            status: EmberStatus::try_decode_from(buffer)?,
        })
    }
}

impl Encode for EmberNetworkInitResponse {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.status.encode_to(buffer);
    }
}

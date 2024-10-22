use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormNetworkResponse {
    pub status: EmberStatus,
}

impl Decode for FormNetworkResponse {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        Ok(Self {
            status: EmberStatus::try_decode_from(buffer)?,
        })
    }
}

impl Encode for FormNetworkResponse {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.status.encode_to(buffer);
    }
}

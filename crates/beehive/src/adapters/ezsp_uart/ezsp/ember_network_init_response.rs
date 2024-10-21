use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberNetworkInitResponse {
    pub status: EmberStatus,
}

impl Decode for EmberNetworkInitResponse {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        Some(Self {
            status: EmberStatus::try_decode_from(buffer).unwrap(),
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

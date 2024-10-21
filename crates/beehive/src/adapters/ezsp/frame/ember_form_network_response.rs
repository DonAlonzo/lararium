use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberFormNetworkResponse {
    pub status: EmberStatus,
}

impl Decode for EmberFormNetworkResponse {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        Some(Self {
            status: EmberStatus::try_decode_from(buffer).unwrap(),
        })
    }
}

impl Encode for EmberFormNetworkResponse {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.status.encode_to(buffer);
    }
}
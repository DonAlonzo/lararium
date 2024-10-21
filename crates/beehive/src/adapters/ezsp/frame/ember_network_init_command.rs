use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberNetworkInitCommand {
    pub bitmask: EmberNetworkInitBitmask,
}

impl Decode for EmberNetworkInitCommand {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        Some(Self {
            bitmask: EmberNetworkInitBitmask::try_decode_from(buffer)?,
        })
    }
}

impl Encode for EmberNetworkInitCommand {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.bitmask.encode_to(buffer);
    }
}

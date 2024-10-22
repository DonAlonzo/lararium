use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkInitCommand {
    pub bitmask: EmberNetworkInitBitmask,
}

impl Decode for NetworkInitCommand {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        Ok(Self {
            bitmask: EmberNetworkInitBitmask::try_decode_from(buffer)?,
        })
    }
}

impl Encode for NetworkInitCommand {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.bitmask.encode_to(buffer);
    }
}

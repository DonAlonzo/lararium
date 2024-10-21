use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberFormNetworkCommand {
    pub parameters: EmberNetworkParameters,
}

impl Decode for EmberFormNetworkCommand {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        Some(Self {
            parameters: EmberNetworkParameters::try_decode_from(buffer)?,
        })
    }
}

impl Encode for EmberFormNetworkCommand {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.parameters.encode_to(buffer);
    }
}

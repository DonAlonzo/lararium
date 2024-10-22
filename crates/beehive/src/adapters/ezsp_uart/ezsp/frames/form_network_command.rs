use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormNetworkCommand {
    pub parameters: EmberNetworkParameters,
}

impl Decode for FormNetworkCommand {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        Ok(Self {
            parameters: EmberNetworkParameters::try_decode_from(buffer)?,
        })
    }
}

impl Encode for FormNetworkCommand {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.parameters.encode_to(buffer);
    }
}

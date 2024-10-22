use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetConfigurationValueCommand {
    pub config_id: EzspConfigId,
}

impl Decode for GetConfigurationValueCommand {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        Ok(Self {
            config_id: EzspConfigId::try_decode_from(buffer)?,
        })
    }
}

impl Encode for GetConfigurationValueCommand {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.config_id.encode_to(buffer);
    }
}

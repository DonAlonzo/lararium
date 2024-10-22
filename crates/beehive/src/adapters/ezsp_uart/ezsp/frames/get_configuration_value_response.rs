use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetConfigurationValueResponse {
    pub status: EzspStatus,
    pub value: u16,
}

impl Decode for GetConfigurationValueResponse {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        let status = EzspStatus::try_decode_from(buffer)?;
        if buffer.remaining() < 2 {
            return Err(DecodeError::InsufficientData);
        }
        let value = buffer.get_u16_le();
        Ok(Self { status, value })
    }
}

impl Encode for GetConfigurationValueResponse {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.status.encode_to(buffer);
        buffer.put_u16_le(self.value);
    }
}

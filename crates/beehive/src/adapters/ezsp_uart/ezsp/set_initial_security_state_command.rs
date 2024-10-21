use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetInitialSecurityStateCommand {
    pub bitmask: EmberInitialSecurityBitmask,
    pub preconfigured_key: EmberKeyData,
    pub network_key: EmberKeyData,
    pub network_key_sequence_number: u8,
    pub preconfigured_trust_center_eui64: EmberEUI64,
}

impl Decode for SetInitialSecurityStateCommand {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        let bitmask = EmberInitialSecurityBitmask::try_decode_from(buffer)?;
        let preconfigured_key = EmberKeyData::try_decode_from(buffer)?;
        let network_key = EmberKeyData::try_decode_from(buffer)?;
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        let network_key_sequence_number = buffer.get_u8();
        let preconfigured_trust_center_eui64 = EmberEUI64::try_decode_from(buffer)?;
        Ok(Self {
            bitmask,
            preconfigured_key,
            network_key,
            network_key_sequence_number,
            preconfigured_trust_center_eui64,
        })
    }
}

impl Encode for SetInitialSecurityStateCommand {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.bitmask.encode_to(buffer);
        self.preconfigured_key.encode_to(buffer);
        self.network_key.encode_to(buffer);
        buffer.put_u8(self.network_key_sequence_number);
        self.preconfigured_trust_center_eui64.encode_to(buffer);
    }
}

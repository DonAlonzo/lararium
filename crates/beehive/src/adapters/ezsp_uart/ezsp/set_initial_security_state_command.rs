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
        Ok(Self {
            bitmask: EmberInitialSecurityBitmask::try_decode_from(buffer)?,
            preconfigured_key: EmberKeyData::try_decode_from(buffer)?,
            network_key: EmberKeyData::try_decode_from(buffer)?,
            network_key_sequence_number: buffer.get_u8(),
            preconfigured_trust_center_eui64: EmberEUI64::try_decode_from(buffer)?,
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

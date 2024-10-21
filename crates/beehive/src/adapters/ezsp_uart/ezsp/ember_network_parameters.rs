use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberNetworkParameters {
    pub extended_pan_id: u64,
    pub pan_id: u16,
    pub radio_tx_power: u8,
    pub radio_channel: u8,
    pub join_method: EmberJoinMethod,
    pub nwk_manager_id: u16,
    pub nwk_update_id: u8,
    pub channels: u32,
}

impl Decode for EmberNetworkParameters {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        Ok(Self {
            extended_pan_id: buffer.get_u64_le(),
            pan_id: buffer.get_u16_le(),
            radio_tx_power: buffer.get_u8(),
            radio_channel: buffer.get_u8(),
            join_method: EmberJoinMethod::try_decode_from(buffer)?,
            nwk_manager_id: buffer.get_u16_le(),
            nwk_update_id: buffer.get_u8(),
            channels: buffer.get_u32_le(),
        })
    }
}

impl Encode for EmberNetworkParameters {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u64_le(self.extended_pan_id);
        buffer.put_u16_le(self.pan_id);
        buffer.put_u8(self.radio_tx_power);
        buffer.put_u8(self.radio_channel);
        self.join_method.encode_to(buffer);
        buffer.put_u16_le(self.nwk_manager_id);
        buffer.put_u8(self.nwk_update_id);
        buffer.put_u32_le(self.channels);
    }
}

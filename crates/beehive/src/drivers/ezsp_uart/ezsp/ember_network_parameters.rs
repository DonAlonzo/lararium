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
        if buffer.remaining() < 12 {
            return Err(DecodeError::InsufficientData);
        }
        let extended_pan_id = buffer.get_u64_le();
        let pan_id = buffer.get_u16_le();
        let radio_tx_power = buffer.get_u8();
        let radio_channel = buffer.get_u8();
        let join_method = EmberJoinMethod::try_decode_from(buffer)?;
        if buffer.remaining() < 7 {
            return Err(DecodeError::InsufficientData);
        }
        let nwk_manager_id = buffer.get_u16_le();
        let nwk_update_id = buffer.get_u8();
        let channels = buffer.get_u32_le();
        Ok(Self {
            extended_pan_id,
            pan_id,
            radio_tx_power,
            radio_channel,
            join_method,
            nwk_manager_id,
            nwk_update_id,
            channels,
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

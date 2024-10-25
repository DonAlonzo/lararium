use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmberApsFrame {
    pub profile_id: u16,
    pub cluster_id: u16,
    pub source_endpoint: u8,
    pub destination_endpoint: u8,
    pub options: EmberApsOption,
    pub group_id: u16,
    pub sequence: u8,
}

impl Decode for EmberApsFrame {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 6 {
            return Err(DecodeError::InsufficientData);
        }
        let profile_id = buffer.get_u16_le();
        let cluster_id = buffer.get_u16_le();
        let source_endpoint = buffer.get_u8();
        let destination_endpoint = buffer.get_u8();
        let options = EmberApsOption::try_decode_from(buffer)?;
        if buffer.remaining() < 3 {
            return Err(DecodeError::InsufficientData);
        }
        let group_id = buffer.get_u16_le();
        let sequence = buffer.get_u8();
        Ok(Self {
            profile_id,
            cluster_id,
            source_endpoint,
            destination_endpoint,
            options,
            group_id,
            sequence,
        })
    }
}

impl Encode for EmberApsFrame {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u16_le(self.profile_id);
        buffer.put_u16_le(self.cluster_id);
        buffer.put_u8(self.source_endpoint);
        buffer.put_u8(self.destination_endpoint);
        self.options.encode_to(buffer);
        buffer.put_u16_le(self.group_id);
        buffer.put_u8(self.sequence);
    }
}

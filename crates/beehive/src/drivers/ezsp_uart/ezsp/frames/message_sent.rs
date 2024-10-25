use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSent {
    pub message_type: EmberOutgoingMessageType,
    pub index_or_destination: u16,
    pub aps_frame: EmberApsFrame,
    pub message_tag: u8,
    pub status: EmberStatus,
    pub message_contents: Vec<u8>,
}

impl Decode for MessageSent {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        let message_type = EmberOutgoingMessageType::try_decode_from(buffer)?;
        if buffer.remaining() < 2 {
            return Err(DecodeError::InsufficientData);
        }
        let index_or_destination = buffer.get_u16_le();
        let aps_frame = EmberApsFrame::try_decode_from(buffer)?;
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        let message_tag = buffer.get_u8();
        let status = EmberStatus::try_decode_from(buffer)?;
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        let message_contents_length = buffer.get_u8() as usize;
        if buffer.remaining() < message_contents_length {
            return Err(DecodeError::InsufficientData);
        }
        let message_contents = buffer.copy_to_bytes(message_contents_length).to_vec();
        Ok(Self {
            message_type,
            index_or_destination,
            aps_frame,
            message_tag,
            status,
            message_contents,
        })
    }
}

impl Encode for MessageSent {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.message_type.encode_to(buffer);
        buffer.put_u16_le(self.index_or_destination);
        self.aps_frame.encode_to(buffer);
        buffer.put_u8(self.message_tag);
        self.status.encode_to(buffer);
        buffer.put_u8(self.message_contents.len() as u8);
        buffer.put_slice(&self.message_contents);
    }
}

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendUnicast {
    pub message_type: EmberOutgoingMessageType,
    pub index_or_destination: EmberNodeId,
    pub aps_frame: EmberApsFrame,
    pub message_tag: u8,
    pub message_contents: Vec<u8>,
}

impl Decode for SendUnicast {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        let message_type = EmberOutgoingMessageType::try_decode_from(buffer)?;
        let index_or_destination = EmberNodeId::try_decode_from(buffer)?;
        let aps_frame = EmberApsFrame::try_decode_from(buffer)?;
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        let message_tag = buffer.get_u8();
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
            message_contents,
        })
    }
}

impl Encode for SendUnicast {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.message_type.encode_to(buffer);
        self.index_or_destination.encode_to(buffer);
        self.aps_frame.encode_to(buffer);
        buffer.put_u8(self.message_tag);
        buffer.put_u8(self.message_contents.len() as u8);
        buffer.put_slice(&self.message_contents);
    }
}

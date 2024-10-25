use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmberOutgoingMessageType {
    Direct,
    ViaAddressTable,
    ViaBinding,
    Multicast,
    Broadcast,
}

impl Decode for EmberOutgoingMessageType {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        use EmberOutgoingMessageType::*;
        Ok(match buffer.get_u8() {
            0x00 => Direct,
            0x01 => ViaAddressTable,
            0x02 => ViaBinding,
            0x03 => Multicast,
            0x04 => Broadcast,
            _ => return Err(DecodeError::Invalid),
        })
    }
}

impl Encode for EmberOutgoingMessageType {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        use EmberOutgoingMessageType::*;
        buffer.put_u8(match self {
            Direct => 0x00,
            ViaAddressTable => 0x01,
            ViaBinding => 0x02,
            Multicast => 0x03,
            Broadcast => 0x04,
        });
    }
}

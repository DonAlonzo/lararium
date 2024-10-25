use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmberNodeType {
    UnkownDevice,
    Coordinator,
    Router,
    EndDevice,
    SleepyEndDevice,
}

impl Decode for EmberNodeType {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        Ok(match buffer.get_u8() {
            0x00 => Self::UnkownDevice,
            0x01 => Self::Coordinator,
            0x02 => Self::Router,
            0x03 => Self::EndDevice,
            0x04 => Self::SleepyEndDevice,
            _ => return Err(DecodeError::Invalid),
        })
    }
}

impl Encode for EmberNodeType {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u8(match self {
            Self::UnkownDevice => 0x00,
            Self::Coordinator => 0x01,
            Self::Router => 0x02,
            Self::EndDevice => 0x03,
            Self::SleepyEndDevice => 0x04,
        });
    }
}

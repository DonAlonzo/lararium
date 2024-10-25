use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmberNetworkStatus {
    NoNetwork,
    JoiningNetwork,
    JoinedNetwork,
    JoinedNetworkNoParent,
    LeavingNetwork,
}

impl Decode for EmberNetworkStatus {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        Ok(match buffer.get_u8() {
            0x00 => Self::NoNetwork,
            0x01 => Self::JoiningNetwork,
            0x02 => Self::JoinedNetwork,
            0x03 => Self::JoinedNetworkNoParent,
            0x04 => Self::LeavingNetwork,
            _ => return Err(DecodeError::Invalid),
        })
    }
}

impl Encode for EmberNetworkStatus {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u8(match self {
            Self::NoNetwork => 0x00,
            Self::JoiningNetwork => 0x01,
            Self::JoinedNetwork => 0x02,
            Self::JoinedNetworkNoParent => 0x03,
            Self::LeavingNetwork => 0x04,
        });
    }
}

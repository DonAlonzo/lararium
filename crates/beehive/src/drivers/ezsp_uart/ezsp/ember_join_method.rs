use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmberJoinMethod {
    UseMacAssociation,
    UseNwkRejoin,
    UseNwkRejoinHaveNwkKey,
    UseConfiguredNwkState,
}

impl Decode for EmberJoinMethod {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 2 {
            return Err(DecodeError::InsufficientData);
        }
        Ok(match buffer.get_u16_le() {
            0x0000 => Self::UseMacAssociation,
            0x0001 => Self::UseNwkRejoin,
            0x0002 => Self::UseNwkRejoinHaveNwkKey,
            0x0003 => Self::UseConfiguredNwkState,
            _ => return Err(DecodeError::Invalid),
        })
    }
}

impl Encode for EmberJoinMethod {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u16_le(match self {
            Self::UseMacAssociation => 0x0000,
            Self::UseNwkRejoin => 0x0001,
            Self::UseNwkRejoinHaveNwkKey => 0x0002,
            Self::UseConfiguredNwkState => 0x0003,
        });
    }
}

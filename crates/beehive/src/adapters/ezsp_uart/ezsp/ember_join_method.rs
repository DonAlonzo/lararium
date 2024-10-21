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
        Ok(match buffer.get_u16() {
            0x00 => Self::UseMacAssociation,
            0x01 => Self::UseNwkRejoin,
            0x02 => Self::UseNwkRejoinHaveNwkKey,
            0x03 => Self::UseConfiguredNwkState,
            _ => return Err(DecodeError::Invalid),
        })
    }
}

impl Encode for EmberJoinMethod {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u16(match self {
            Self::UseMacAssociation => 0x00,
            Self::UseNwkRejoin => 0x01,
            Self::UseNwkRejoinHaveNwkKey => 0x02,
            Self::UseConfiguredNwkState => 0x03,
        });
    }
}

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EzspStatus {
    Success,
    VersionNotSet,
}

impl Decode for EzspStatus {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        use EzspStatus::*;
        Ok(match buffer.get_u8() {
            0x00 => Success,
            0x30 => VersionNotSet,
            _ => return Err(DecodeError::Invalid),
        })
    }
}

impl Encode for EzspStatus {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        use EzspStatus::*;
        buffer.put_u8(match self {
            Success => 0x00,
            VersionNotSet => 0x30,
        });
    }
}

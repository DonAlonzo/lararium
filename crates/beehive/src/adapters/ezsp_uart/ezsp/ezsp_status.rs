use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EzspStatus {
    Success,
    VersionNotSet,
}

impl Decode for EzspStatus {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        if buffer.remaining() < 1 {
            return None;
        }
        Some(match buffer.get_u8() {
            0x00 => Self::Success,
            0x30 => Self::VersionNotSet,
            _ => return None,
        })
    }
}

impl Encode for EzspStatus {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u8(match self {
            Self::Success => 0x00,
            Self::VersionNotSet => 0x30,
        });
    }
}

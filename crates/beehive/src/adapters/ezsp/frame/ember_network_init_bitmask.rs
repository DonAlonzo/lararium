use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmberNetworkInitBitmask {
    NoOptions,
    ParentInfoInToken,
    EndDeviceRejoinOnReboot,
}

impl Decode for EmberNetworkInitBitmask {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        Some(match buffer.get_u16() {
            0x0000 => Self::NoOptions,
            0x0001 => Self::ParentInfoInToken,
            0x0002 => Self::EndDeviceRejoinOnReboot,
            _ => return None,
        })
    }
}

impl Encode for EmberNetworkInitBitmask {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u16(match self {
            Self::NoOptions => 0x0000,
            Self::ParentInfoInToken => 0x0001,
            Self::EndDeviceRejoinOnReboot => 0x0002,
        });
    }
}
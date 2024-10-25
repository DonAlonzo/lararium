use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberNetworkInitBitmask(u16);

pub enum EmberNetworkInitBitmaskFlag {
    ParentInfoInToken,
    EndDeviceRejoinOnReboot,
}

impl EmberNetworkInitBitmask {
    pub fn new(flags: &[EmberNetworkInitBitmaskFlag]) -> Self {
        let mut bitmask = 0;
        use EmberNetworkInitBitmaskFlag::*;
        for flag in flags {
            bitmask |= match flag {
                ParentInfoInToken => 0x0001,
                EndDeviceRejoinOnReboot => 0x0002,
            }
        }
        Self(bitmask)
    }
}

impl Decode for EmberNetworkInitBitmask {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 2 {
            return Err(DecodeError::InsufficientData);
        }
        Ok(Self(buffer.get_u16_le()))
    }
}

impl Encode for EmberNetworkInitBitmask {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u16_le(self.0);
    }
}

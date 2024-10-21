use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberInitialSecurityBitmask(u16);

impl EmberInitialSecurityBitmask {
    pub fn new() -> Self {
        Self(0)
    }
}

impl Decode for EmberInitialSecurityBitmask {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        if buffer.remaining() < 2 {
            return None;
        }
        Some(Self(buffer.get_u16_le()))
    }
}

impl Encode for EmberInitialSecurityBitmask {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u16_le(self.0);
    }
}

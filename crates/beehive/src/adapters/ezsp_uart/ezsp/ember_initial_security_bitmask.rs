use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberInitialSecurityBitmask(u16);

pub enum EmberInitialSecurityBitmaskFlag {
    StandardSecurityMode,
    DistributedTrustCenterMode,
    TrustCenterGlobalLinkKey,
    PreconfiguredNetworkKeyMode,
    TrustCenterUsesHashedLinkKey,
    HavePreconfiguredKey,
    HaveNetworkKey,
    GetLinkKeyWhenJoining,
    RequireEncryptedKey,
    NoFrameCounterReset,
    GetPreconfiguredKeyFromInstallCode,
    HaveTrustCenterEui64,
}

impl EmberInitialSecurityBitmask {
    pub fn new(flags: &[EmberInitialSecurityBitmaskFlag]) -> Self {
        let mut bitmask = 0;
        use EmberInitialSecurityBitmaskFlag::*;
        for flag in flags {
            bitmask |= match flag {
                StandardSecurityMode => 0x0000,
                DistributedTrustCenterMode => 0x0002,
                TrustCenterGlobalLinkKey => 0x0004,
                PreconfiguredNetworkKeyMode => 0x0008,
                TrustCenterUsesHashedLinkKey => 0x0084,
                HavePreconfiguredKey => 0x0100,
                HaveNetworkKey => 0x0200,
                GetLinkKeyWhenJoining => 0x0400,
                RequireEncryptedKey => 0x0800,
                NoFrameCounterReset => 0x1000,
                GetPreconfiguredKeyFromInstallCode => 0x2000,
                HaveTrustCenterEui64 => 0x0040,
            }
        }
        Self(bitmask)
    }
}

impl Decode for EmberInitialSecurityBitmask {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 2 {
            return Err(DecodeError::InsufficientData);
        }
        Ok(Self(buffer.get_u16_le()))
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

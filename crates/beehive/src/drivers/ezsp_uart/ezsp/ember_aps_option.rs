use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberApsOption(u16);

pub enum EmberApsOptionFlag {
    Encryption,
    Retry,
    EnableRouteDiscovery,
    ForceRouteDiscovery,
    SourceEui64,
    DestinationEui64,
    EnableAddressDiscovery,
    PollResponse,
    ZdoResponseRequired,
    Fragment,
}

impl EmberApsOption {
    pub fn new(flags: &[EmberApsOptionFlag]) -> Self {
        let mut bitmask = 0;
        use EmberApsOptionFlag::*;
        for flag in flags {
            bitmask |= match flag {
                Encryption => 0x0020,
                Retry => 0x0040,
                EnableRouteDiscovery => 0x0100,
                ForceRouteDiscovery => 0x0200,
                SourceEui64 => 0x0400,
                DestinationEui64 => 0x0800,
                EnableAddressDiscovery => 0x1000,
                PollResponse => 0x2000,
                ZdoResponseRequired => 0x4000,
                Fragment => 0x8000,
            }
        }
        Self(bitmask)
    }
}

impl Decode for EmberApsOption {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 2 {
            return Err(DecodeError::InsufficientData);
        }
        Ok(Self(buffer.get_u16_le()))
    }
}

impl Encode for EmberApsOption {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u16_le(self.0);
    }
}

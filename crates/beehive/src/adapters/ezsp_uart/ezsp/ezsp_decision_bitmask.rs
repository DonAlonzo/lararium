use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EzspDecisionBitmask(u16);

pub enum EzspDecisionBitmaskFlag {
    AllowJoins,
    AllowUnsecuredRejoins,
    SendKeyInClear,
    IgnoreUnsecuredRejoins,
    JoinsUseInstallCodeKey,
    DeferJoins,
}

impl EzspDecisionBitmask {
    pub fn new(flags: &[EzspDecisionBitmaskFlag]) -> Self {
        let mut bitmask = 0;
        use EzspDecisionBitmaskFlag::*;
        for flag in flags {
            bitmask |= match flag {
                AllowJoins => 0x0001,
                AllowUnsecuredRejoins => 0x0002,
                SendKeyInClear => 0x0004,
                IgnoreUnsecuredRejoins => 0x0008,
                JoinsUseInstallCodeKey => 0x0010,
                DeferJoins => 0x0020,
            }
        }
        Self(bitmask)
    }
}

impl Decode for EzspDecisionBitmask {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 2 {
            return Err(DecodeError::InsufficientData);
        }
        Ok(Self(buffer.get_u16_le()))
    }
}

impl Encode for EzspDecisionBitmask {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u16_le(self.0);
    }
}

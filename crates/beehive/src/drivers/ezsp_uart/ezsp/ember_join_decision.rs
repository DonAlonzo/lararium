use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmberJoinDecision {
    UsePreconfiguredKey,
    SendKeyInTheClear,
    DenyJoin,
    NoAction,
}

impl Decode for EmberJoinDecision {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        use EmberJoinDecision::*;
        Ok(match buffer.get_u8() {
            0x00 => UsePreconfiguredKey,
            0x01 => SendKeyInTheClear,
            0x02 => DenyJoin,
            0x03 => NoAction,
            _ => return Err(DecodeError::Invalid),
        })
    }
}

impl Encode for EmberJoinDecision {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        use EmberJoinDecision::*;
        buffer.put_u8(match self {
            UsePreconfiguredKey => 0x00,
            SendKeyInTheClear => 0x01,
            DenyJoin => 0x02,
            NoAction => 0x03,
        });
    }
}

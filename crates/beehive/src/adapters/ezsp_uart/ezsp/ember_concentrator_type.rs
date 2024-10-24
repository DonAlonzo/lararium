use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmberConcentratorType {
    LowRamConcentrator,
    HighRamConcentrator,
}

impl Decode for EmberConcentratorType {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 2 {
            return Err(DecodeError::InsufficientData);
        }
        use EmberConcentratorType::*;
        Ok(match buffer.get_u16_le() {
            0xFFF8 => LowRamConcentrator,
            0xFFF9 => HighRamConcentrator,
            _ => return Err(DecodeError::Invalid),
        })
    }
}

impl Encode for EmberConcentratorType {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        use EmberConcentratorType::*;
        buffer.put_u16_le(match self {
            LowRamConcentrator => 0xFFF8,
            HighRamConcentrator => 0xFFF9,
        });
    }
}

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmberStatus {
    Success,
    FatalError,
    InvalidCall,
    NetworkUp,
    NotJoined,
    SecurityStateNotSet,
    SecurityConfigurationInvalid,
}

impl Decode for EmberStatus {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        use EmberStatus::*;
        Ok(match buffer.get_u8() {
            0x00 => Success,
            0x01 => FatalError,
            0x70 => InvalidCall,
            0x90 => NetworkUp,
            0x93 => NotJoined,
            0xA8 => SecurityStateNotSet,
            0xB7 => SecurityConfigurationInvalid,
            code => panic!("unknown status: {code:02X}"),
        })
    }
}

impl Encode for EmberStatus {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        use EmberStatus::*;
        let byte = match self {
            Success => 0x00,
            FatalError => 0x01,
            InvalidCall => 0x70,
            NetworkUp => 0x90,
            NotJoined => 0x93,
            SecurityStateNotSet => 0xA8,
            SecurityConfigurationInvalid => 0xB7,
        };
        buffer.put_u8(byte);
    }
}

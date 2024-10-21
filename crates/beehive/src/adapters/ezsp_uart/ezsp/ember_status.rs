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
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        if buffer.remaining() < 1 {
            return None;
        }
        use EmberStatus::*;
        match buffer.get_u8() {
            0x00 => Some(Success),
            0x01 => Some(FatalError),
            0x70 => Some(InvalidCall),
            0x90 => Some(NetworkUp),
            0x93 => Some(NotJoined),
            0xA8 => Some(SecurityStateNotSet),
            0xB7 => Some(SecurityConfigurationInvalid),
            code => panic!("unknown status: {code:02X}"),
        }
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

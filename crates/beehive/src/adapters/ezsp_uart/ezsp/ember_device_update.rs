use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmberDeviceUpdate {
    StandardSecuritySecuredRejoin,
    StandardSecurityUnsecuredJoin,
    DeviceLeft,
    StandardSecurityUnsecuredRejoin,
}

impl Decode for EmberDeviceUpdate {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        use EmberDeviceUpdate::*;
        Ok(match buffer.get_u8() {
            0x00 => StandardSecuritySecuredRejoin,
            0x01 => StandardSecurityUnsecuredJoin,
            0x02 => DeviceLeft,
            0x03 => StandardSecurityUnsecuredRejoin,
            _ => return Err(DecodeError::Invalid),
        })
    }
}

impl Encode for EmberDeviceUpdate {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        use EmberDeviceUpdate::*;
        buffer.put_u8(match self {
            StandardSecuritySecuredRejoin => 0x00,
            StandardSecurityUnsecuredJoin => 0x01,
            DeviceLeft => 0x02,
            StandardSecurityUnsecuredRejoin => 0x03,
        });
    }
}

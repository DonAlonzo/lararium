use super::*;

// https://github.com/SiliconLabs/gecko_sdk/blob/gsdk_4.4/app/zcl/manufacturers.xml

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Manufacturer {
    IkeaOfSweden,
    Philips,
}

impl Decode for Manufacturer {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 2 {
            return Err(DecodeError::InsufficientData);
        }
        use Manufacturer::*;
        Ok(match buffer.get_u16_le() {
            0x117C => IkeaOfSweden,
            0x10CB => Philips,
            _ => return Err(DecodeError::Invalid),
        })
    }
}

impl Encode for Manufacturer {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        use Manufacturer::*;
        buffer.put_u16_le(match self {
            IkeaOfSweden => 0x117C,
            Philips => 0x10CB,
        });
    }
}

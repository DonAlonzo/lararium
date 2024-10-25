use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EzspExtendedValueId {
    EndpointFlags,
    LastLeaveReason,
    GetSourceRouteOverhead,
}

impl Decode for EzspExtendedValueId {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        use EzspExtendedValueId::*;
        Ok(match buffer.get_u8() {
            0x00 => EndpointFlags,
            0x01 => LastLeaveReason,
            0x02 => GetSourceRouteOverhead,
            _ => return Err(DecodeError::Invalid),
        })
    }
}

impl Encode for EzspExtendedValueId {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        use EzspExtendedValueId::*;
        buffer.put_u8(match self {
            EndpointFlags => 0x00,
            LastLeaveReason => 0x01,
            GetSourceRouteOverhead => 0x02,
        });
    }
}

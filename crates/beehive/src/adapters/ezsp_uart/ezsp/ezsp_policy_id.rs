use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EzspPolicyId {
    TrustCenterPolicy,
    BindingModificationPolicy,
    UnicastRepliesPolicy,
    PollHandlerPolicy,
    MessageContentsInCallbackPolicy,
    TcKeyRequestPolicy,
    AppKeyRequestPolicy,
    PacketValidateLibraryPolicy,
    ZllPolicy,
    TcRejoinsUsingWellKnownKeyPolicy,
}

impl Decode for EzspPolicyId {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        use EzspPolicyId::*;
        Ok(match buffer.get_u8() {
            0x00 => TrustCenterPolicy,
            0x01 => BindingModificationPolicy,
            0x02 => UnicastRepliesPolicy,
            0x03 => PollHandlerPolicy,
            0x04 => MessageContentsInCallbackPolicy,
            0x05 => TcKeyRequestPolicy,
            0x06 => AppKeyRequestPolicy,
            0x07 => PacketValidateLibraryPolicy,
            0x08 => ZllPolicy,
            0x09 => TcRejoinsUsingWellKnownKeyPolicy,
            _ => return Err(DecodeError::Invalid),
        })
    }
}

impl Encode for EzspPolicyId {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        use EzspPolicyId::*;
        buffer.put_u8(match self {
            TrustCenterPolicy => 0x00,
            BindingModificationPolicy => 0x01,
            UnicastRepliesPolicy => 0x02,
            PollHandlerPolicy => 0x03,
            MessageContentsInCallbackPolicy => 0x04,
            TcKeyRequestPolicy => 0x05,
            AppKeyRequestPolicy => 0x06,
            PacketValidateLibraryPolicy => 0x07,
            ZllPolicy => 0x08,
            TcRejoinsUsingWellKnownKeyPolicy => 0x09,
        })
    }
}

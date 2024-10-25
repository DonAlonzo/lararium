use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EzspDecisionId {
    DeferJoinsRejoinsHaveLinkKey,
    DisallowBindingModification,
    AllowBindingModification,
    CheckBindingModificationsAreValidEndpointClusters,
    HostWillNotSupplyReply,
    HostWillSupplyReply,
    PollHandlerIgnore,
    PollHandlerCallback,
    MessageTagOnlyInCallback,
    MessageTagAndContentsInCallback,
    DenyTcKeyRequests,
    AllowTcKeyRequestsAndSendCurrentKey,
    AllowTcKeyRequestAndGenerateNewKey,
    DenyAppKeyRequests,
    AllowAppKeyRequests,
    PacketValidateLibraryChecksEnabled,
    PacketValidateLibraryChecksDisabled,
}

impl Decode for EzspDecisionId {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        use EzspDecisionId::*;
        Ok(match buffer.get_u8() {
            0x07 => DeferJoinsRejoinsHaveLinkKey,
            0x10 => DisallowBindingModification,
            0x11 => AllowBindingModification,
            0x12 => CheckBindingModificationsAreValidEndpointClusters,
            0x20 => HostWillNotSupplyReply,
            0x21 => HostWillSupplyReply,
            0x30 => PollHandlerIgnore,
            0x31 => PollHandlerCallback,
            0x40 => MessageTagOnlyInCallback,
            0x41 => MessageTagAndContentsInCallback,
            0x50 => DenyTcKeyRequests,
            0x51 => AllowTcKeyRequestsAndSendCurrentKey,
            0x52 => AllowTcKeyRequestAndGenerateNewKey,
            0x60 => DenyAppKeyRequests,
            0x61 => AllowAppKeyRequests,
            0x62 => PacketValidateLibraryChecksEnabled,
            0x63 => PacketValidateLibraryChecksDisabled,
            _ => return Err(DecodeError::Invalid),
        })
    }
}

impl Encode for EzspDecisionId {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        use EzspDecisionId::*;
        buffer.put_u8(match self {
            DeferJoinsRejoinsHaveLinkKey => 0x07,
            DisallowBindingModification => 0x10,
            AllowBindingModification => 0x11,
            CheckBindingModificationsAreValidEndpointClusters => 0x12,
            HostWillNotSupplyReply => 0x20,
            HostWillSupplyReply => 0x21,
            PollHandlerIgnore => 0x30,
            PollHandlerCallback => 0x31,
            MessageTagOnlyInCallback => 0x40,
            MessageTagAndContentsInCallback => 0x41,
            DenyTcKeyRequests => 0x50,
            AllowTcKeyRequestsAndSendCurrentKey => 0x51,
            AllowTcKeyRequestAndGenerateNewKey => 0x52,
            DenyAppKeyRequests => 0x60,
            AllowAppKeyRequests => 0x61,
            PacketValidateLibraryChecksEnabled => 0x62,
            PacketValidateLibraryChecksDisabled => 0x63,
        });
    }
}

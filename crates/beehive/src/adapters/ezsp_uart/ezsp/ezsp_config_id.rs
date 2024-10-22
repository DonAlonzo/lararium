use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EzspConfigId {
    PacketBufferCount,
    NeighborTableSize,
    ApsUnicastMessageCount,
    BindingTableSize,
    AddressTableSize,
    MulticastTableSize,
    RouteTableSize,
    DiscoveryTableSize,
    StackProfile,
    SecurityLevel,
    MaxHops,
    MaxEndDeviceChildren,
    IndirectTransmissionTimeout,
    EndDevicePollTimeout,
    TxPowerMode,
    DisableRelay,
    TrustCenterAddressCacheSize,
    SourceRouteTableSize,
    FragmentWindowSize,
    FragmentDelayMs,
    KeyTableSize,
    ApsAckTimeout,
    BeaconJitterDuration,
    PanIdConflictReportThreshold,
    RequestKeyTimeout,
    CertificateTableSize,
    ApplicationZdoFlags,
    BroadcastTableSize,
    MacFilterTableSize,
    SupportedNetworks,
    SendMulticastsToSleepyAddress,
    ZllGroupAddresses,
    ZllRssiThreshold,
    MtorrFlowControl,
    RetryQueueSize,
    NewBroadcastEntryThreshold,
    TransientKeyTimeoutS,
    BroadcastMinAcksNeeded,
    TcRejoinsUsingWellKnownKeyTimeoutS,
    CtuneValue,
    AssumeTcConcentratorType,
    GpProxyTableSize,
    GpSinkTableSize,
}

impl Decode for EzspConfigId {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        if buffer.remaining() < 1 {
            return Err(DecodeError::InsufficientData);
        }
        use EzspConfigId::*;
        Ok(match buffer.get_u8() {
            0x01 => PacketBufferCount,
            0x02 => NeighborTableSize,
            0x03 => ApsUnicastMessageCount,
            0x04 => BindingTableSize,
            0x05 => AddressTableSize,
            0x06 => MulticastTableSize,
            0x07 => RouteTableSize,
            0x08 => DiscoveryTableSize,
            0x0C => StackProfile,
            0x0D => SecurityLevel,
            0x10 => MaxHops,
            0x11 => MaxEndDeviceChildren,
            0x12 => IndirectTransmissionTimeout,
            0x13 => EndDevicePollTimeout,
            0x17 => TxPowerMode,
            0x18 => DisableRelay,
            0x19 => TrustCenterAddressCacheSize,
            0x1A => SourceRouteTableSize,
            0x1C => FragmentWindowSize,
            0x1D => FragmentDelayMs,
            0x1E => KeyTableSize,
            0x1F => ApsAckTimeout,
            0x20 => BeaconJitterDuration,
            0x22 => PanIdConflictReportThreshold,
            0x24 => RequestKeyTimeout,
            0x29 => CertificateTableSize,
            0x2A => ApplicationZdoFlags,
            0x2B => BroadcastTableSize,
            0x2C => MacFilterTableSize,
            0x2D => SupportedNetworks,
            0x2E => SendMulticastsToSleepyAddress,
            0x2F => ZllGroupAddresses,
            0x30 => ZllRssiThreshold,
            0x33 => MtorrFlowControl,
            0x34 => RetryQueueSize,
            0x35 => NewBroadcastEntryThreshold,
            0x36 => TransientKeyTimeoutS,
            0x37 => BroadcastMinAcksNeeded,
            0x38 => TcRejoinsUsingWellKnownKeyTimeoutS,
            0x39 => CtuneValue,
            0x40 => AssumeTcConcentratorType,
            0x41 => GpProxyTableSize,
            0x42 => GpSinkTableSize,
            _ => return Err(DecodeError::Invalid),
        })
    }
}

impl Encode for EzspConfigId {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        use EzspConfigId::*;
        buffer.put_u8(match self {
            PacketBufferCount => 0x01,
            NeighborTableSize => 0x02,
            ApsUnicastMessageCount => 0x03,
            BindingTableSize => 0x04,
            AddressTableSize => 0x05,
            MulticastTableSize => 0x06,
            RouteTableSize => 0x07,
            DiscoveryTableSize => 0x08,
            StackProfile => 0x0C,
            SecurityLevel => 0x0D,
            MaxHops => 0x10,
            MaxEndDeviceChildren => 0x11,
            IndirectTransmissionTimeout => 0x12,
            EndDevicePollTimeout => 0x13,
            TxPowerMode => 0x17,
            DisableRelay => 0x18,
            TrustCenterAddressCacheSize => 0x19,
            SourceRouteTableSize => 0x1A,
            FragmentWindowSize => 0x1C,
            FragmentDelayMs => 0x1D,
            KeyTableSize => 0x1E,
            ApsAckTimeout => 0x1F,
            BeaconJitterDuration => 0x20,
            PanIdConflictReportThreshold => 0x22,
            RequestKeyTimeout => 0x24,
            CertificateTableSize => 0x29,
            ApplicationZdoFlags => 0x2A,
            BroadcastTableSize => 0x2B,
            MacFilterTableSize => 0x2C,
            SupportedNetworks => 0x2D,
            SendMulticastsToSleepyAddress => 0x2E,
            ZllGroupAddresses => 0x2F,
            ZllRssiThreshold => 0x30,
            MtorrFlowControl => 0x33,
            RetryQueueSize => 0x34,
            NewBroadcastEntryThreshold => 0x35,
            TransientKeyTimeoutS => 0x36,
            BroadcastMinAcksNeeded => 0x37,
            TcRejoinsUsingWellKnownKeyTimeoutS => 0x38,
            CtuneValue => 0x39,
            AssumeTcConcentratorType => 0x40,
            GpProxyTableSize => 0x41,
            GpSinkTableSize => 0x42,
        });
    }
}

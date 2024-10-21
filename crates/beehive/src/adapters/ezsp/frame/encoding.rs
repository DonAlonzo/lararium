use super::*;
use bytes::BufMut;

pub trait Decode: Sized {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self>;
}

pub trait Encode {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    );
}

/***********************/

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

/***********************/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberEUI64([u8; 8]);

impl EmberEUI64 {
    pub fn new(value: [u8; 8]) -> Self {
        Self(value)
    }
}

impl Decode for EmberEUI64 {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        if buffer.remaining() < 8 {
            return None;
        }
        let mut value = [0; 8];
        buffer.copy_to_slice(&mut value);
        Some(Self(value))
    }
}

impl Encode for EmberEUI64 {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_slice(&self.0);
    }
}

/***********************/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberKeyData([u8; 16]);

impl EmberKeyData {
    pub fn new(data: [u8; 16]) -> Self {
        Self(data)
    }
}

impl Decode for EmberKeyData {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        if buffer.remaining() < 16 {
            return None;
        }
        let mut data = [0; 16];
        buffer.copy_to_slice(&mut data);
        Some(Self(data))
    }
}

impl Encode for EmberKeyData {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_slice(&self.0);
    }
}

/***********************/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberInitialSecurityBitmask(u16);

impl EmberInitialSecurityBitmask {
    pub fn new() -> Self {
        Self(0)
    }
}

impl Decode for EmberInitialSecurityBitmask {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        if buffer.remaining() < 2 {
            return None;
        }
        Some(Self(buffer.get_u16_le()))
    }
}

impl Encode for EmberInitialSecurityBitmask {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        buffer.put_u16_le(self.0);
    }
}

/***********************/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetInitialSecurityStateCommand {
    pub bitmask: EmberInitialSecurityBitmask,
    pub preconfigured_key: EmberKeyData,
    pub network_key: EmberKeyData,
    pub network_key_sequence_number: u8,
    pub preconfigured_trust_center_eui64: EmberEUI64,
}

impl Decode for SetInitialSecurityStateCommand {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        Some(Self {
            bitmask: EmberInitialSecurityBitmask::try_decode_from(buffer)?,
            preconfigured_key: EmberKeyData::try_decode_from(buffer)?,
            network_key: EmberKeyData::try_decode_from(buffer)?,
            network_key_sequence_number: buffer.get_u8(),
            preconfigured_trust_center_eui64: EmberEUI64::try_decode_from(buffer)?,
        })
    }
}

impl Encode for SetInitialSecurityStateCommand {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.bitmask.encode_to(buffer);
        self.preconfigured_key.encode_to(buffer);
        self.network_key.encode_to(buffer);
        buffer.put_u8(self.network_key_sequence_number);
        self.preconfigured_trust_center_eui64.encode_to(buffer);
    }
}

/***********************/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetInitialSecurityStateResponse {
    pub status: EmberStatus,
}

impl Decode for SetInitialSecurityStateResponse {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        Some(Self {
            status: EmberStatus::try_decode_from(buffer)?,
        })
    }
}

impl Encode for SetInitialSecurityStateResponse {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.status.encode_to(buffer);
    }
}

/***********************/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackStatusHandlerResponse {
    pub status: EmberStatus,
}

impl Decode for StackStatusHandlerResponse {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Option<Self> {
        Some(Self {
            status: EmberStatus::try_decode_from(buffer)?,
        })
    }
}

impl Encode for StackStatusHandlerResponse {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        self.status.encode_to(buffer);
    }
}

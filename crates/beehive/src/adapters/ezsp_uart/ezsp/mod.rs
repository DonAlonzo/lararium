mod frames;
pub use frames::*;

mod ember_eui64;
pub use ember_eui64::*;
mod ember_initial_security_bitmask;
pub use ember_initial_security_bitmask::*;
mod ember_initial_security_state;
pub use ember_initial_security_state::*;
mod ember_join_method;
pub use ember_join_method::*;
mod ember_key_data;
pub use ember_key_data::*;
mod ember_network_init_bitmask;
pub use ember_network_init_bitmask::*;
mod ember_network_parameters;
pub use ember_network_parameters::*;
mod ember_status;
pub use ember_status::*;
mod ezsp_config_id;
pub use ezsp_config_id::*;
mod ezsp_status;
pub use ezsp_status::*;
mod frame_id;
pub use frame_id::*;
mod primitives;
pub use primitives::*;

use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    Invalid,
    InsufficientData,
}

impl std::error::Error for DecodeError {}

impl std::fmt::Display for DecodeError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub trait Decode: Sized {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError>;
}

pub trait Encode {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameVersion1Command {
    pub sequence: u8,
    pub network_index: u8,
    pub sleep_mode: SleepMode,
    pub security_enabled: bool,
    pub padding_enabled: bool,
    pub frame_id: FrameId,
    pub parameters: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameVersion1Response {
    pub sequence: u8,
    pub network_index: u8,
    pub callback_type: CallbackType,
    pub pending: bool,
    pub truncated: bool,
    pub overflow: bool,
    pub security_enabled: bool,
    pub padding_enabled: bool,
    pub frame_id: FrameId,
    pub parameters: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameVersion0Command {
    pub sequence: u8,
    pub network_index: u8,
    pub sleep_mode: SleepMode,
    pub frame_id: FrameId,
    pub parameters: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameVersion0Response {
    pub sequence: u8,
    pub network_index: u8,
    pub callback_type: CallbackType,
    pub pending: bool,
    pub truncated: bool,
    pub overflow: bool,
    pub frame_id: FrameId,
    pub parameters: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepMode {
    PowerDown,
    DeepSleep,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackType {
    Asynchronous,
    Synchronous,
    None,
}

impl Encode for FrameVersion1Command {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        let frame_control_low = {
            let mut byte = 0x00;
            byte |= (self.network_index & 0b11) << 5;
            byte | match self.sleep_mode {
                SleepMode::PowerDown => 0b0000_0010,
                SleepMode::DeepSleep => 0b0000_0001,
                SleepMode::Idle => 0b0000_0000,
            }
        };
        let frame_control_high = {
            let mut byte = 0x00;
            if self.security_enabled {
                byte |= 0b1000_0000;
            }
            if self.padding_enabled {
                byte |= 0b0100_0000;
            }
            // Version
            byte |= 0b0000_0001;
            byte
        };
        buffer.put_u8(self.sequence);
        buffer.put_u8(frame_control_low);
        buffer.put_u8(frame_control_high);
        buffer.put_u16_le(self.frame_id.into());
        buffer.put_slice(&self.parameters);
    }
}

impl Decode for FrameVersion1Command {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        let sequence = buffer.get_u8();
        let frame_control_low = buffer.get_u8();
        let is_command = (frame_control_low & 0b1000_0000) == 0;
        if !is_command {
            return Err(DecodeError::Invalid);
        }
        let network_index = (frame_control_low & 0b0110_0000) >> 5;
        let frame_control_high = buffer.get_u8();
        let security_enabled = frame_control_high & 0b1000_0000 != 0;
        let padding_enabled = frame_control_high & 0b0100_0000 != 0;
        if frame_control_high & 0b0000_0001 == 0 {
            return Err(DecodeError::Invalid);
        }
        let frame_id: FrameId = buffer.get_u16_le().try_into().unwrap();
        let parameters = buffer.copy_to_bytes(buffer.remaining()).to_vec();
        let sleep_mode = match frame_control_low & 0b0000_0011 {
            0b10 => SleepMode::PowerDown,
            0b01 => SleepMode::DeepSleep,
            0b00 => SleepMode::Idle,
            _ => panic!("unknown sleep mode"),
        };
        Ok(Self {
            sequence,
            network_index,
            sleep_mode,
            padding_enabled,
            security_enabled,
            frame_id,
            parameters,
        })
    }
}

impl Encode for FrameVersion1Response {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        let frame_control_low = {
            let mut byte = 0x00;
            byte |= (self.network_index & 0b11) << 5;
            byte |= match self.callback_type {
                CallbackType::Asynchronous => 0b0001_0000,
                CallbackType::Synchronous => 0b0000_1000,
                CallbackType::None => 0b0000_0010,
            };
            if self.pending {
                byte |= 0b0000_0100;
            }
            if self.truncated {
                byte |= 0b0000_0010;
            }
            if self.overflow {
                byte |= 0b0000_0001;
            }
            byte
        };
        let frame_control_high = {
            let mut byte = 0x00;
            if self.security_enabled {
                byte |= 0b1000_0000;
            }
            if self.padding_enabled {
                byte |= 0b0100_0000;
            }
            // Version
            byte |= 0b0000_0001;
            byte
        };
        buffer.put_u8(self.sequence);
        buffer.put_u8(frame_control_low);
        buffer.put_u8(frame_control_high);
        buffer.put_u16_le(self.frame_id.into());
        buffer.put_slice(&self.parameters);
    }
}

impl Decode for FrameVersion1Response {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        let sequence = buffer.get_u8();
        let frame_control_low = buffer.get_u8();
        let is_command = (frame_control_low & 0b1000_0000) == 0;
        if is_command {
            return Err(DecodeError::Invalid);
        }
        let network_index = (frame_control_low & 0b0110_0000) >> 5;
        let frame_control_high = buffer.get_u8();
        let security_enabled = frame_control_high & 0b1000_0000 != 0;
        let padding_enabled = frame_control_high & 0b0100_0000 != 0;
        if frame_control_high & 0b0000_0001 == 0 {
            return Err(DecodeError::Invalid);
        }
        let frame_id: FrameId = buffer.get_u16_le().try_into().unwrap();
        let parameters = buffer.copy_to_bytes(buffer.remaining()).to_vec();
        let callback_type = match (frame_control_low >> 3) & 0b11 {
            0b10 => CallbackType::Asynchronous,
            0b01 => CallbackType::Synchronous,
            0b00 => CallbackType::None,
            _ => panic!("unknown callback type"),
        };
        let pending = (frame_control_low >> 2) & 0b1 != 0;
        let truncated = (frame_control_low >> 1) & 0b1 != 0;
        let overflow = frame_control_low & 0b1 != 0;
        Ok(Self {
            sequence,
            network_index,
            callback_type,
            pending,
            truncated,
            overflow,
            padding_enabled,
            security_enabled,
            frame_id,
            parameters,
        })
    }
}

impl Encode for FrameVersion0Command {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        let frame_control_low = {
            let mut byte = 0x00;
            byte |= (self.network_index & 0b11) << 5;
            byte | match self.sleep_mode {
                SleepMode::PowerDown => 0b0000_0010,
                SleepMode::DeepSleep => 0b0000_0001,
                SleepMode::Idle => 0b0000_0000,
            }
        };
        let frame_id: u16 = self.frame_id.into();
        buffer.put_u8(self.sequence);
        buffer.put_u8(frame_control_low);
        buffer.put_u8(frame_id as u8);
        buffer.put_slice(&self.parameters);
    }
}

impl Decode for FrameVersion0Command {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        let sequence = buffer.get_u8();
        let frame_control_low = buffer.get_u8();
        let is_command = (frame_control_low & 0b1000_0000) == 0;
        if !is_command {
            return Err(DecodeError::Invalid);
        }
        let network_index = (frame_control_low & 0b0110_0000) >> 5;
        let frame_id: FrameId = (buffer.get_u8() as u16).try_into().unwrap();
        let parameters = buffer.copy_to_bytes(buffer.remaining()).to_vec();
        let sleep_mode = match frame_control_low & 0b0000_0011 {
            0b10 => SleepMode::PowerDown,
            0b01 => SleepMode::DeepSleep,
            0b00 => SleepMode::Idle,
            value => panic!("unknown sleep mode: {value:b}"),
        };
        Ok(Self {
            sequence,
            network_index,
            sleep_mode,
            frame_id,
            parameters,
        })
    }
}

impl Encode for FrameVersion0Response {
    fn encode_to<B: BufMut>(
        &self,
        buffer: &mut B,
    ) {
        let frame_control_low = {
            let mut byte = 0x00;
            byte |= (self.network_index & 0b11) << 5;
            byte |= match self.callback_type {
                CallbackType::Asynchronous => 0b0001_0000,
                CallbackType::Synchronous => 0b0000_1000,
                CallbackType::None => 0b0000_0010,
            };
            if self.pending {
                byte |= 0b0000_0100;
            }
            if self.truncated {
                byte |= 0b0000_0010;
            }
            if self.overflow {
                byte |= 0b0000_0001;
            }
            byte
        };
        let frame_id = {
            let frame_id: u16 = self.frame_id.into();
            if frame_id > 0xFF {
                panic!("bad frame id");
            }
            frame_id as u8
        };
        buffer.put_u8(self.sequence);
        buffer.put_u8(frame_control_low);
        buffer.put_u8(frame_id);
        buffer.put_slice(&self.parameters);
    }
}

impl Decode for FrameVersion0Response {
    fn try_decode_from<B: Buf>(buffer: &mut B) -> Result<Self, DecodeError> {
        let sequence = buffer.get_u8();
        let frame_control_low = buffer.get_u8();
        let is_command = (frame_control_low & 0b1000_0000) == 0;
        if is_command {
            return Err(DecodeError::Invalid);
        }
        let network_index = (frame_control_low & 0b0110_0000) >> 5;
        let frame_id: FrameId = (buffer.get_u8() as u16).try_into().unwrap();
        let parameters = buffer.copy_to_bytes(buffer.remaining()).to_vec();
        let callback_type = match (frame_control_low >> 3) & 0b11 {
            0b10 => CallbackType::Asynchronous,
            0b01 => CallbackType::Synchronous,
            0b00 => CallbackType::None,
            value => panic!("unknown callback type: {value:b}"),
        };
        let pending = (frame_control_low >> 2) & 0b1 != 0;
        let truncated = (frame_control_low >> 1) & 0b1 != 0;
        let overflow = frame_control_low & 0b1 != 0;
        Ok(Self {
            sequence,
            network_index,
            callback_type,
            pending,
            truncated,
            overflow,
            frame_id,
            parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_version_response() {
        let mut bytes = Bytes::from_static(&[0x00, 0x80, 0x00, 0x0D, 0x02, 0x30, 0x74]);
        let actual = FrameVersion0Response::try_decode_from(&mut bytes).unwrap();
        let expected = FrameVersion0Response {
            sequence: 0,
            network_index: 0b00,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            frame_id: FrameId::Version,
            parameters: {
                let mut buffer = BytesMut::new();
                let response = VersionResponse {
                    protocol_version: 13,
                    stack_type: 2,
                    stack_version: 29744,
                };
                response.encode_to(&mut buffer);
                buffer.to_vec()
            },
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_unknown_command_response() {
        let mut bytes = Bytes::from_static(&[0x01, 0x80, 0x01, 0x58, 0x00, 0x30]);
        let actual = FrameVersion1Response::try_decode_from(&mut bytes).unwrap();
        let expected = FrameVersion1Response {
            sequence: 1,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            frame_id: FrameId::InvalidCommand,
            parameters: {
                let mut buffer = BytesMut::new();
                let response = InvalidCommandResponse {
                    status: EzspStatus::VersionNotSet,
                };
                response.encode_to(&mut buffer);
                buffer.to_vec()
            },
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_network_init_response() {
        let mut bytes = Bytes::from_static(&[0x01, 0x80, 0x01, 0x17, 0x00, 0x93]);
        let actual = FrameVersion1Response::try_decode_from(&mut bytes).unwrap();
        let expected = FrameVersion1Response {
            sequence: 1,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            frame_id: FrameId::NetworkInit,
            parameters: {
                let mut buffer = BytesMut::new();
                let response = NetworkInitResponse {
                    status: EmberStatus::NotJoined,
                };
                response.encode_to(&mut buffer);
                buffer.to_vec()
            },
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_form_network_response_1() {
        let mut bytes = Bytes::from_static(&[0x02, 0x80, 0x01, 0x1E, 0x00, 0x00]);
        let actual = FrameVersion1Response::try_decode_from(&mut bytes).unwrap();
        let expected = FrameVersion1Response {
            sequence: 2,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            frame_id: FrameId::FormNetwork,
            parameters: {
                let mut buffer = BytesMut::new();
                let response = FormNetworkResponse {
                    status: EmberStatus::Success,
                };
                response.encode_to(&mut buffer);
                buffer.to_vec()
            },
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_form_network_response_2() {
        let mut bytes = Bytes::from_static(&[0x02, 0x80, 0x01, 0x1E, 0x00, 0xA8]);
        let actual = FrameVersion1Response::try_decode_from(&mut bytes).unwrap();
        let expected = FrameVersion1Response {
            sequence: 2,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            frame_id: FrameId::FormNetwork,
            parameters: {
                let mut buffer = BytesMut::new();
                let response = FormNetworkResponse {
                    status: EmberStatus::SecurityStateNotSet,
                };
                response.encode_to(&mut buffer);
                buffer.to_vec()
            },
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_form_network_response_3() {
        let mut bytes = Bytes::from_static(&[0x03, 0x80, 0x01, 0x1E, 0x00, 0xA8]);
        let actual = FrameVersion1Response::try_decode_from(&mut bytes).unwrap();
        let expected = FrameVersion1Response {
            sequence: 3,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            frame_id: FrameId::FormNetwork,
            parameters: {
                let mut buffer = BytesMut::new();
                let response = FormNetworkResponse {
                    status: EmberStatus::SecurityStateNotSet,
                };
                response.encode_to(&mut buffer);
                buffer.to_vec()
            },
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_form_network_response_4() {
        let mut bytes = Bytes::from_static(&[0x03, 0x84, 0x01, 0x1E, 0x00, 0x00]);
        let actual = FrameVersion1Response::try_decode_from(&mut bytes).unwrap();
        let expected = FrameVersion1Response {
            sequence: 3,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: true,
            truncated: false,
            overflow: false,
            frame_id: FrameId::FormNetwork,
            parameters: {
                let mut buffer = BytesMut::new();
                let response = FormNetworkResponse {
                    status: EmberStatus::Success,
                };
                response.encode_to(&mut buffer);
                buffer.to_vec()
            },
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_form_network_response_5() {
        let mut bytes = Bytes::from_static(&[0x03, 0x90, 0x01, 0x1E, 0x00, 0x70]);
        let actual = FrameVersion1Response::try_decode_from(&mut bytes).unwrap();
        let expected = FrameVersion1Response {
            sequence: 3,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::Asynchronous,
            pending: false,
            truncated: false,
            overflow: false,
            frame_id: FrameId::FormNetwork,
            parameters: {
                let mut buffer = BytesMut::new();
                let response = FormNetworkResponse {
                    status: EmberStatus::InvalidCall,
                };
                response.encode_to(&mut buffer);
                buffer.to_vec()
            },
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_set_initial_security_state_response() {
        let mut bytes = Bytes::from_static(&[0x02, 0x80, 0x01, 0x68, 0x00, 0xB7]);
        let actual = FrameVersion1Response::try_decode_from(&mut bytes).unwrap();
        let expected = FrameVersion1Response {
            sequence: 2,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            frame_id: FrameId::SetInitialSecurityState,
            parameters: {
                let mut buffer = BytesMut::new();
                let response = SetInitialSecurityStateResponse {
                    status: EmberStatus::SecurityConfigurationInvalid,
                };
                response.encode_to(&mut buffer);
                buffer.to_vec()
            },
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_stack_status_handler_response() {
        let mut bytes = Bytes::from_static(&[0x03, 0x90, 0x01, 0x19, 0x00, 0x90]);
        let actual = FrameVersion1Response::try_decode_from(&mut bytes).unwrap();
        let expected = FrameVersion1Response {
            sequence: 3,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::Asynchronous,
            pending: false,
            truncated: false,
            overflow: false,
            frame_id: FrameId::StackStatusHandler,
            parameters: {
                let mut buffer = BytesMut::new();
                let response = StackStatusHandlerResponse {
                    status: EmberStatus::NetworkUp,
                };
                response.encode_to(&mut buffer);
                buffer.to_vec()
            },
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_get_configuration_value_response() {
        let mut bytes = Bytes::from_static(&[0x04, 0x80, 0x01, 0x52, 0x00, 0x00, 0x05, 0x00]);
        let actual = FrameVersion1Response::try_decode_from(&mut bytes).unwrap();
        let expected = FrameVersion1Response {
            sequence: 4,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            frame_id: FrameId::GetConfigurationValue,
            parameters: {
                let mut buffer = BytesMut::new();
                let response = GetConfigurationValueResponse {
                    status: EzspStatus::Success,
                    value: 5,
                };
                response.encode_to(&mut buffer);
                buffer.to_vec()
            },
        };
        assert_eq!(expected, actual);
    }
}

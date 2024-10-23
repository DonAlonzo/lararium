mod frames;
pub use frames::*;

mod ember_eui64;
pub use ember_eui64::*;
mod ember_initial_security_bitmask;
pub use ember_initial_security_bitmask::*;
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
mod empty;
pub use empty::*;
mod ezsp_config_id;
pub use ezsp_config_id::*;
mod ezsp_status;
pub use ezsp_status::*;
mod frame_id;
pub use frame_id::*;

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
pub enum FrameVersion1 {
    Command {
        sequence: u8,
        network_index: u8,
        sleep_mode: SleepMode,
        security_enabled: bool,
        padding_enabled: bool,
        frame_id: FrameId,
        parameters: Vec<u8>,
    },
    Response {
        sequence: u8,
        network_index: u8,
        callback_type: CallbackType,
        pending: bool,
        truncated: bool,
        overflow: bool,
        security_enabled: bool,
        padding_enabled: bool,
        frame_id: FrameId,
        parameters: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameVersion0 {
    Command {
        sequence: u8,
        network_index: u8,
        sleep_mode: SleepMode,
        frame_id: FrameId,
        parameters: Vec<u8>,
    },
    Response {
        sequence: u8,
        network_index: u8,
        callback_type: CallbackType,
        pending: bool,
        truncated: bool,
        overflow: bool,
        frame_id: FrameId,
        parameters: Vec<u8>,
    },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Version(VersionCommand),
    Callback,
    NetworkInit(NetworkInitCommand),
    FormNetwork(FormNetworkCommand),
    GetConfigurationValue(GetConfigurationValueCommand),
    SetInitialSecurityState(SetInitialSecurityStateCommand),
    PermitJoining(PermitJoiningCommand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Response {
    Version(VersionResponse),
    NoCallback,
    NetworkInit(NetworkInitResponse),
    StackStatusHandler(StackStatusHandlerResponse),
    FormNetwork(FormNetworkResponse),
    GetConfigurationValue(GetConfigurationValueResponse),
    InvalidCommand(InvalidCommandResponse),
    SetInitialSecurityState(SetInitialSecurityStateResponse),
    PermitJoining(PermitJoiningResponse),
}

impl FrameVersion1 {
    pub fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        match self {
            FrameVersion1::Command {
                sequence,
                network_index,
                sleep_mode,
                security_enabled,
                padding_enabled,
                frame_id,
                parameters,
            } => {
                let frame_control_low = {
                    let mut byte = 0x00;
                    byte |= (network_index & 0b11) << 5;
                    byte | match sleep_mode {
                        SleepMode::PowerDown => 0b0000_0010,
                        SleepMode::DeepSleep => 0b0000_0001,
                        SleepMode::Idle => 0b0000_0000,
                    }
                };
                let frame_control_high = {
                    let mut byte = 0x00;
                    if *security_enabled {
                        byte |= 0b1000_0000;
                    }
                    if *padding_enabled {
                        byte |= 0b0100_0000;
                    }
                    // Version
                    byte |= 0b0000_0001;
                    byte
                };
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_control_high);
                buffer.put_u16_le((*frame_id).into());
                buffer.put_slice(&parameters);
            }
            FrameVersion1::Response {
                sequence,
                network_index,
                callback_type,
                pending,
                truncated,
                overflow,
                security_enabled,
                padding_enabled,
                frame_id,
                parameters,
            } => {
                let frame_control_low = {
                    let mut byte = 0x00;
                    byte |= (network_index & 0b11) << 5;
                    byte |= match callback_type {
                        CallbackType::Asynchronous => 0b0001_0000,
                        CallbackType::Synchronous => 0b0000_1000,
                        CallbackType::None => 0b0000_0010,
                    };
                    if *pending {
                        byte |= 0b0000_0100;
                    }
                    if *truncated {
                        byte |= 0b0000_0010;
                    }
                    if *overflow {
                        byte |= 0b0000_0001;
                    }
                    byte
                };
                let frame_control_high = {
                    let mut byte = 0x00;
                    if *security_enabled {
                        byte |= 0b1000_0000;
                    }
                    if *padding_enabled {
                        byte |= 0b0100_0000;
                    }
                    // Version
                    byte |= 0b0000_0001;
                    byte
                };
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_control_high);
                buffer.put_u16_le((*frame_id).into());
                buffer.put_slice(&parameters);
            }
        }
        buffer.freeze()
    }

    pub fn decode(bytes: &mut Bytes) -> Result<Self, DecodeError> {
        let sequence = bytes.get_u8();
        let frame_control_low = bytes.get_u8();
        let is_command = (frame_control_low & 0b1000_0000) == 0;
        let network_index = (frame_control_low & 0b0110_0000) >> 5;
        let frame_control_high = bytes.get_u8();
        let security_enabled = frame_control_high & 0b1000_0000 != 0;
        let padding_enabled = frame_control_high & 0b0100_0000 != 0;
        if frame_control_high & 0b0000_0001 == 0 {
            return Err(DecodeError::Invalid);
        }
        let frame_id: FrameId = bytes.get_u16_le().try_into().unwrap();
        let mut parameters = bytes.to_vec();
        if is_command {
            let sleep_mode = match frame_control_low & 0b0000_0011 {
                0b10 => SleepMode::PowerDown,
                0b01 => SleepMode::DeepSleep,
                0b00 => SleepMode::Idle,
                _ => panic!("unknown sleep mode"),
            };
            Ok(Self::Command {
                sequence,
                network_index,
                sleep_mode,
                padding_enabled,
                security_enabled,
                frame_id,
                parameters,
            })
        } else {
            let callback_type = match (frame_control_low >> 3) & 0b11 {
                0b10 => CallbackType::Asynchronous,
                0b01 => CallbackType::Synchronous,
                0b00 => CallbackType::None,
                _ => panic!("unknown callback type"),
            };
            let pending = (frame_control_low >> 2) & 0b1 != 0;
            let truncated = (frame_control_low >> 1) & 0b1 != 0;
            let overflow = frame_control_low & 0b1 != 0;
            Ok(Self::Response {
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
}

impl FrameVersion0 {
    pub fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        match self {
            FrameVersion0::Command {
                sequence,
                network_index,
                sleep_mode,
                frame_id,
                parameters,
            } => {
                let frame_control_low = {
                    let mut byte = 0x00;
                    byte |= (network_index & 0b11) << 5;
                    byte | match sleep_mode {
                        SleepMode::PowerDown => 0b0000_0010,
                        SleepMode::DeepSleep => 0b0000_0001,
                        SleepMode::Idle => 0b0000_0000,
                    }
                };
                let frame_id: u16 = (*frame_id).into();
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_id as u8);
                buffer.put_slice(&parameters);
            }
            FrameVersion0::Response {
                sequence,
                network_index,
                callback_type,
                pending,
                truncated,
                overflow,
                frame_id,
                parameters,
            } => {
                let frame_control_low = {
                    let mut byte = 0x00;
                    byte |= (network_index & 0b11) << 5;
                    byte |= match callback_type {
                        CallbackType::Asynchronous => 0b0001_0000,
                        CallbackType::Synchronous => 0b0000_1000,
                        CallbackType::None => 0b0000_0010,
                    };
                    if *pending {
                        byte |= 0b0000_0100;
                    }
                    if *truncated {
                        byte |= 0b0000_0010;
                    }
                    if *overflow {
                        byte |= 0b0000_0001;
                    }
                    byte
                };
                let frame_id = {
                    let frame_id: u16 = (*frame_id).into();
                    if frame_id > 0xFF {
                        panic!("bad frame id");
                    }
                    frame_id as u8
                };
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_id);
                buffer.put_slice(&parameters);
            }
        }
        buffer.freeze()
    }

    pub fn decode(bytes: &mut Bytes) -> Result<Self, DecodeError> {
        let sequence = bytes.get_u8();
        let frame_control_low = bytes.get_u8();
        let is_command = (frame_control_low & 0b1000_0000) == 0;
        let network_index = (frame_control_low & 0b0110_0000) >> 5;
        let frame_id: FrameId = (bytes.get_u8() as u16).try_into().unwrap();
        let parameters = bytes.to_vec();
        if is_command {
            let sleep_mode = match frame_control_low & 0b0000_0011 {
                0b10 => SleepMode::PowerDown,
                0b01 => SleepMode::DeepSleep,
                0b00 => SleepMode::Idle,
                value => panic!("unknown sleep mode: {value:b}"),
            };
            Ok(Self::Command {
                sequence,
                network_index,
                sleep_mode,
                frame_id,
                parameters,
            })
        } else {
            let callback_type = match (frame_control_low >> 3) & 0b11 {
                0b10 => CallbackType::Asynchronous,
                0b01 => CallbackType::Synchronous,
                0b00 => CallbackType::None,
                value => panic!("unknown callback type: {value:b}"),
            };
            let pending = (frame_control_low >> 2) & 0b1 != 0;
            let truncated = (frame_control_low >> 1) & 0b1 != 0;
            let overflow = frame_control_low & 0b1 != 0;
            Ok(Self::Response {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_version_response() {
        let mut bytes = Bytes::from_static(&[0x00, 0x80, 0x00, 0x0D, 0x02, 0x30, 0x74]);
        let actual = FrameVersion0::decode(&mut bytes).unwrap();
        let expected = FrameVersion0::Response {
            sequence: 0,
            network_index: 0b00,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            response: Response::Version(VersionResponse {
                protocol_version: 13,
                stack_type: 2,
                stack_version: 29744,
            }),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_unknown_command_response() {
        let mut bytes = Bytes::from_static(&[0x01, 0x80, 0x01, 0x58, 0x00, 0x30]);
        let actual = FrameVersion1::decode(&mut bytes).unwrap();
        let expected = FrameVersion1::Response {
            sequence: 1,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            response: Response::InvalidCommand(InvalidCommandResponse {
                status: EzspStatus::VersionNotSet,
            }),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_network_init_response() {
        let mut bytes = Bytes::from_static(&[0x01, 0x80, 0x01, 0x17, 0x00, 0x93]);
        let actual = FrameVersion1::decode(&mut bytes).unwrap();
        let expected = FrameVersion1::Response {
            sequence: 1,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            response: Response::NetworkInit(NetworkInitResponse {
                status: EmberStatus::NotJoined,
            }),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_form_network_response_1() {
        let mut bytes = Bytes::from_static(&[0x02, 0x80, 0x01, 0x1E, 0x00, 0x00]);
        let actual = FrameVersion1::decode(&mut bytes).unwrap();
        let expected = FrameVersion1::Response {
            sequence: 2,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            response: Response::FormNetwork(FormNetworkResponse {
                status: EmberStatus::Success,
            }),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_form_network_response_2() {
        let mut bytes = Bytes::from_static(&[0x02, 0x80, 0x01, 0x1E, 0x00, 0xA8]);
        let actual = FrameVersion1::decode(&mut bytes).unwrap();
        let expected = FrameVersion1::Response {
            sequence: 2,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            response: Response::FormNetwork(FormNetworkResponse {
                status: EmberStatus::SecurityStateNotSet,
            }),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_form_network_response_3() {
        let mut bytes = Bytes::from_static(&[0x03, 0x80, 0x01, 0x1E, 0x00, 0xA8]);
        let actual = FrameVersion1::decode(&mut bytes).unwrap();
        let expected = FrameVersion1::Response {
            sequence: 3,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            response: Response::FormNetwork(FormNetworkResponse {
                status: EmberStatus::SecurityStateNotSet,
            }),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_form_network_response_4() {
        let mut bytes = Bytes::from_static(&[0x03, 0x84, 0x01, 0x1E, 0x00, 0x00]);
        let actual = FrameVersion1::decode(&mut bytes).unwrap();
        let expected = FrameVersion1::Response {
            sequence: 3,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: true,
            truncated: false,
            overflow: false,
            response: Response::FormNetwork(FormNetworkResponse {
                status: EmberStatus::Success,
            }),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_form_network_response_5() {
        let mut bytes = Bytes::from_static(&[0x03, 0x90, 0x01, 0x1E, 0x00, 0x70]);
        let actual = FrameVersion1::decode(&mut bytes).unwrap();
        let expected = FrameVersion1::Response {
            sequence: 3,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::Asynchronous,
            pending: false,
            truncated: false,
            overflow: false,
            response: Response::FormNetwork(FormNetworkResponse {
                status: EmberStatus::InvalidCall,
            }),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_set_initial_security_state_response() {
        let mut bytes = Bytes::from_static(&[0x02, 0x80, 0x01, 0x68, 0x00, 0xB7]);
        let actual = FrameVersion1::decode(&mut bytes).unwrap();
        let expected = FrameVersion1::Response {
            sequence: 2,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            response: Response::SetInitialSecurityState(SetInitialSecurityStateResponse {
                status: EmberStatus::SecurityConfigurationInvalid,
            }),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_stack_status_handler_response() {
        let mut bytes = Bytes::from_static(&[0x03, 0x90, 0x01, 0x19, 0x00, 0x90]);
        let actual = FrameVersion1::decode(&mut bytes).unwrap();
        let expected = FrameVersion1::Response {
            sequence: 3,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::Asynchronous,
            pending: false,
            truncated: false,
            overflow: false,
            response: Response::StackStatusHandler(StackStatusHandlerResponse {
                status: EmberStatus::NetworkUp,
            }),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_get_configuration_value_response() {
        let mut bytes = Bytes::from_static(&[0x04, 0x80, 0x01, 0x52, 0x00, 0x00, 0x05, 0x00]);
        let actual = FrameVersion1::decode(&mut bytes).unwrap();
        let expected = FrameVersion1::Response {
            sequence: 4,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::None,
            pending: false,
            truncated: false,
            overflow: false,
            response: Response::GetConfigurationValue(GetConfigurationValueResponse {
                status: EzspStatus::Success,
                value: 5,
            }),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_() {
        let mut bytes = Bytes::from_static(&[0x05, 0x90, 0x01, 0x19, 0x00, 0x9C]);
        let actual = FrameVersion1::decode(&mut bytes).unwrap();
        let expected = FrameVersion1::Response {
            sequence: 5,
            network_index: 0b00,
            padding_enabled: false,
            security_enabled: false,
            callback_type: CallbackType::Asynchronous,
            pending: false,
            truncated: false,
            overflow: false,
            response: Response::StackStatusHandler(StackStatusHandlerResponse {
                status: EmberStatus::NetworkOpened,
            }),
        };
        assert_eq!(expected, actual);
    }
}

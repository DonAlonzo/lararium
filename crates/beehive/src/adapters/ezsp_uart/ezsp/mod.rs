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
mod ezsp_config_id;
pub use ezsp_config_id::*;
mod ezsp_status;
pub use ezsp_status::*;
mod frame_id;
use frame_id::*;

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
        command: Command,
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
        response: Response,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameVersion0 {
    Command {
        sequence: u8,
        network_index: u8,
        sleep_mode: SleepMode,
        command: Command,
    },
    Response {
        sequence: u8,
        network_index: u8,
        callback_type: CallbackType,
        pending: bool,
        truncated: bool,
        overflow: bool,
        response: Response,
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
                command,
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
                let frame_id = match command {
                    Command::Version(_) => 0x0000,
                    Command::Callback => 0x0006,
                    Command::NetworkInit(_) => 0x0017,
                    Command::FormNetwork(_) => 0x001E,
                    Command::GetConfigurationValue(_) => 0x0052,
                    Command::SetInitialSecurityState(_) => 0x0068,
                    Command::PermitJoining(_) => 0x0022,
                };
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_control_high);
                buffer.put_u16_le(frame_id);
                match command {
                    Command::Version(command) => command.encode_to(&mut buffer),
                    Command::Callback => (),
                    Command::NetworkInit(command) => command.encode_to(&mut buffer),
                    Command::FormNetwork(command) => command.encode_to(&mut buffer),
                    Command::GetConfigurationValue(command) => command.encode_to(&mut buffer),
                    Command::SetInitialSecurityState(command) => command.encode_to(&mut buffer),
                    Command::PermitJoining(command) => command.encode_to(&mut buffer),
                };
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
                response,
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
                let frame_id = match response {
                    Response::Version(_) => 0x0000,
                    Response::NoCallback => 0x0007,
                    Response::NetworkInit(_) => 0x0017,
                    Response::StackStatusHandler(_) => 0x0019,
                    Response::FormNetwork(_) => 0x001E,
                    Response::GetConfigurationValue(_) => 0x0052,
                    Response::InvalidCommand(_) => 0x0058,
                    Response::SetInitialSecurityState(_) => 0x0068,
                    Response::PermitJoining(_) => 0x0022,
                };
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_control_high);
                buffer.put_u16_le(frame_id);
                match response {
                    Response::Version(response) => response.encode_to(&mut buffer),
                    Response::NoCallback => (),
                    Response::NetworkInit(response) => response.encode_to(&mut buffer),
                    Response::StackStatusHandler(response) => response.encode_to(&mut buffer),
                    Response::FormNetwork(response) => response.encode_to(&mut buffer),
                    Response::GetConfigurationValue(response) => response.encode_to(&mut buffer),
                    Response::InvalidCommand(response) => response.encode_to(&mut buffer),
                    Response::SetInitialSecurityState(response) => response.encode_to(&mut buffer),
                    Response::PermitJoining(response) => response.encode_to(&mut buffer),
                };
            }
        }
        buffer.freeze()
    }

    pub fn decode(bytes: &mut Bytes) -> Self {
        let sequence = bytes.get_u8();
        let frame_control_low = bytes.get_u8();
        let is_command = (frame_control_low & 0b1000_0000) == 0;
        let network_index = (frame_control_low & 0b0110_0000) >> 5;
        let frame_control_high = bytes.get_u8();
        let security_enabled = frame_control_high & 0b1000_0000 != 0;
        let padding_enabled = frame_control_high & 0b0100_0000 != 0;
        if frame_control_high & 0b0000_0001 == 0 {
            panic!("unknown frame format version");
        }
        let frame_id = bytes.get_u16_le();
        let mut parameters = Bytes::from(bytes.to_vec());
        if is_command {
            let sleep_mode = match frame_control_low & 0b0000_0011 {
                0b10 => SleepMode::PowerDown,
                0b01 => SleepMode::DeepSleep,
                0b00 => SleepMode::Idle,
                _ => panic!("unknown sleep mode"),
            };
            let command = match frame_id {
                0x0000 => {
                    Command::Version(VersionCommand::try_decode_from(&mut parameters).unwrap())
                }
                0x0017 => Command::NetworkInit(
                    NetworkInitCommand::try_decode_from(&mut parameters).unwrap(),
                ),
                _ => panic!("unknown command"),
            };
            Self::Command {
                sequence,
                network_index,
                sleep_mode,
                padding_enabled,
                security_enabled,
                command,
            }
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
            let response = match frame_id {
                0x0000 => {
                    Response::Version(VersionResponse::try_decode_from(&mut parameters).unwrap())
                }
                0x0007 => Response::NoCallback,
                0x0017 => Response::NetworkInit(
                    NetworkInitResponse::try_decode_from(&mut parameters).unwrap(),
                ),
                0x0019 => Response::StackStatusHandler(
                    StackStatusHandlerResponse::try_decode_from(&mut parameters).unwrap(),
                ),
                0x001E => Response::FormNetwork(
                    FormNetworkResponse::try_decode_from(&mut parameters).unwrap(),
                ),
                0x0022 => Response::PermitJoining(
                    PermitJoiningResponse::try_decode_from(&mut parameters).unwrap(),
                ),
                0x0052 => Response::GetConfigurationValue(
                    GetConfigurationValueResponse::try_decode_from(&mut parameters).unwrap(),
                ),
                0x0058 => Response::InvalidCommand(
                    InvalidCommandResponse::try_decode_from(&mut parameters).unwrap(),
                ),
                0x0068 => Response::SetInitialSecurityState(
                    SetInitialSecurityStateResponse::try_decode_from(&mut parameters).unwrap(),
                ),
                _ => panic!("unknown frame id: {frame_id:02X}"),
            };
            Self::Response {
                sequence,
                network_index,
                callback_type,
                pending,
                truncated,
                overflow,
                padding_enabled,
                security_enabled,
                response,
            }
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
                command,
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
                let frame_id = match command {
                    Command::Version(_) => 0x00,
                    Command::Callback => 0x06,
                    Command::NetworkInit(_) => 0x17,
                    Command::FormNetwork(_) => 0x1E,
                    Command::GetConfigurationValue(_) => 0x52,
                    Command::SetInitialSecurityState(_) => 0x68,
                    Command::PermitJoining(_) => 0x22,
                };
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_id);
                match command {
                    Command::Version(command) => command.encode_to(&mut buffer),
                    Command::Callback => (),
                    Command::NetworkInit(command) => command.encode_to(&mut buffer),
                    Command::FormNetwork(command) => command.encode_to(&mut buffer),
                    Command::GetConfigurationValue(command) => command.encode_to(&mut buffer),
                    Command::SetInitialSecurityState(command) => command.encode_to(&mut buffer),
                    Command::PermitJoining(command) => command.encode_to(&mut buffer),
                };
            }
            FrameVersion0::Response {
                sequence,
                network_index,
                callback_type,
                pending,
                truncated,
                overflow,
                response,
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
                    use FrameId::*;
                    let frame_id = match response {
                        Response::Version(_) => Version,
                        Response::NoCallback => NoCallback,
                        Response::NetworkInit(_) => NetworkInit,
                        Response::StackStatusHandler(_) => StackStatusHandler,
                        Response::FormNetwork(_) => FormNetwork,
                        Response::GetConfigurationValue(_) => GetConfigurationValue,
                        Response::InvalidCommand(_) => InvalidCommand,
                        Response::SetInitialSecurityState(_) => SetInitialSecurityState,
                        Response::PermitJoining(_) => PermitJoining,
                    } as u16;
                    if frame_id > 0xFF {
                        panic!("unsupported frame id")
                    }
                    frame_id as u8
                };
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_id);
                match response {
                    Response::Version(response) => response.encode_to(&mut buffer),
                    Response::NoCallback => (),
                    Response::NetworkInit(response) => response.encode_to(&mut buffer),
                    Response::StackStatusHandler(response) => response.encode_to(&mut buffer),
                    Response::FormNetwork(response) => response.encode_to(&mut buffer),
                    Response::GetConfigurationValue(response) => response.encode_to(&mut buffer),
                    Response::InvalidCommand(response) => response.encode_to(&mut buffer),
                    Response::SetInitialSecurityState(response) => response.encode_to(&mut buffer),
                    Response::PermitJoining(response) => response.encode_to(&mut buffer),
                };
            }
        }
        buffer.freeze()
    }

    pub fn decode(bytes: &mut Bytes) -> Self {
        let sequence = bytes.get_u8();
        let frame_control_low = bytes.get_u8();
        let is_command = (frame_control_low & 0b1000_0000) == 0;
        let network_index = (frame_control_low & 0b0110_0000) >> 5;
        let frame_id: FrameId = (bytes.get_u8() as u16).try_into().unwrap();
        let mut parameters = Bytes::from(bytes.to_vec());
        if is_command {
            let sleep_mode = match frame_control_low & 0b0000_0011 {
                0b10 => SleepMode::PowerDown,
                0b01 => SleepMode::DeepSleep,
                0b00 => SleepMode::Idle,
                value => panic!("unknown sleep mode: {value:b}"),
            };
            let command = match frame_id {
                FrameId::Version => {
                    Command::Version(VersionCommand::try_decode_from(&mut parameters).unwrap())
                }
                FrameId::NetworkInit => Command::NetworkInit(
                    NetworkInitCommand::try_decode_from(&mut parameters).unwrap(),
                ),
                _ => panic!("unknown command: {frame_id}"),
            };
            Self::Command {
                sequence,
                network_index,
                sleep_mode,
                command,
            }
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
            let response = match frame_id {
                FrameId::Version => Response::Version({
                    VersionResponse::try_decode_from(&mut parameters).unwrap()
                }),
                FrameId::NetworkInit => Response::NetworkInit(
                    NetworkInitResponse::try_decode_from(&mut parameters).unwrap(),
                ),
                FrameId::FormNetwork => Response::FormNetwork(
                    FormNetworkResponse::try_decode_from(&mut parameters).unwrap(),
                ),
                FrameId::GetConfigurationValue => Response::GetConfigurationValue(
                    GetConfigurationValueResponse::try_decode_from(&mut parameters).unwrap(),
                ),
                FrameId::InvalidCommand => Response::InvalidCommand(
                    InvalidCommandResponse::try_decode_from(&mut parameters).unwrap(),
                ),
                FrameId::SetInitialSecurityState => Response::SetInitialSecurityState({
                    SetInitialSecurityStateResponse::try_decode_from(&mut parameters).unwrap()
                }),
                _ => panic!("unknown frame id: {frame_id}"),
            };
            Self::Response {
                sequence,
                network_index,
                callback_type,
                pending,
                truncated,
                overflow,
                response,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_version_response() {
        let mut bytes = Bytes::from_static(&[0x00, 0x80, 0x00, 0x0D, 0x02, 0x30, 0x74]);
        let actual = FrameVersion0::decode(&mut bytes);
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
        let actual = FrameVersion1::decode(&mut bytes);
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
        let actual = FrameVersion1::decode(&mut bytes);
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
        let actual = FrameVersion1::decode(&mut bytes);
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
        let actual = FrameVersion1::decode(&mut bytes);
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
        let actual = FrameVersion1::decode(&mut bytes);
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
        let actual = FrameVersion1::decode(&mut bytes);
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
        let actual = FrameVersion1::decode(&mut bytes);
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
        let actual = FrameVersion1::decode(&mut bytes);
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
        let actual = FrameVersion1::decode(&mut bytes);
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
        let actual = FrameVersion1::decode(&mut bytes);
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
}

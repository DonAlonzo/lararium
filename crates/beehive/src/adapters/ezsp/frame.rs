use bytes::{Buf, BufMut, Bytes, BytesMut};

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
pub enum FrameFormatVersion {
    Version1,
    Version0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Version(EmberVersionCommand),
    NetworkInit(EmberNetworkInitCommand),
    FormNetwork(EmberFormNetworkCommand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Response {
    Version(EmberVersionResponse),
    NetworkInit(EmberNetworkInitResponse),
    FormNetwork(EmberFormNetworkResponse),
    UnknownCommand(UnknownCommandResponse),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberVersionCommand {
    desired_protocol_version: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberVersionResponse {
    protocol_version: u8,
    stack_type: u8,
    stack_version: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownCommandResponse {
    status: EzspStatus,
}

impl EmberNetworkInitCommand {
    pub fn encode(&self) -> Bytes {
        self.bitmask.encode()
    }

    pub fn decode(bytes: &mut Bytes) -> Self {
        Self {
            bitmask: EmberNetworkInitBitmask::decode(bytes),
        }
    }
}

impl EmberNetworkInitBitmask {
    fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u16(match self {
            EmberNetworkInitBitmask::NoOptions => 0x0000,
            EmberNetworkInitBitmask::ParentInfoInToken => 0x0001,
            EmberNetworkInitBitmask::EndDeviceRejoinOnReboot => 0x0002,
        });
        buffer.freeze()
    }

    fn decode(bytes: &mut Bytes) -> Self {
        match bytes.get_u16() {
            0x0000 => EmberNetworkInitBitmask::NoOptions,
            0x0001 => EmberNetworkInitBitmask::ParentInfoInToken,
            0x0002 => EmberNetworkInitBitmask::EndDeviceRejoinOnReboot,
            _ => panic!("unknown bitmask"),
        }
    }
}

impl EmberNetworkInitResponse {
    fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u8(self.status.encode());
        buffer.freeze()
    }

    fn decode(bytes: &mut Bytes) -> Self {
        Self {
            status: EmberStatus::decode(bytes.get_u8()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberNetworkInitCommand {
    pub bitmask: EmberNetworkInitBitmask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberNetworkInitResponse {
    pub status: EmberStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmberNetworkInitBitmask {
    NoOptions,
    ParentInfoInToken,
    EndDeviceRejoinOnReboot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberFormNetworkCommand {
    pub parameters: EmberNetworkParameters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberFormNetworkResponse {
    pub status: EmberStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmberNetworkParameters {
    pub extended_pan_id: u64,
    pub pan_id: u16,
    pub radio_tx_power: u8,
    pub radio_channel: u8,
    pub join_method: EmberJoinMethod,
    pub nwk_manager_id: u16,
    pub nwk_update_id: u8,
    pub channels: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmberJoinMethod {
    UseMacAssociation,
    UseNwkRejoin,
    UseNwkRejoinHaveNwkKey,
    UseConfiguredNwkState,
}

impl EmberFormNetworkCommand {
    pub fn encode(&self) -> Bytes {
        self.parameters.encode()
    }

    pub fn decode(bytes: &mut Bytes) -> Self {
        Self {
            parameters: EmberNetworkParameters::decode(bytes),
        }
    }
}

impl EmberFormNetworkResponse {
    fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u8(self.status.encode());
        buffer.freeze()
    }

    fn decode(bytes: &mut Bytes) -> Self {
        Self {
            status: EmberStatus::decode(bytes.get_u8()),
        }
    }
}

impl EmberNetworkParameters {
    fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u64_le(self.extended_pan_id);
        buffer.put_u16_le(self.pan_id);
        buffer.put_u8(self.radio_tx_power);
        buffer.put_u8(self.radio_channel);
        buffer.put_u8(self.join_method.encode());
        buffer.put_u16_le(self.nwk_manager_id);
        buffer.put_u8(self.nwk_update_id);
        buffer.put_u32_le(self.channels);
        buffer.freeze()
    }

    fn decode(bytes: &mut Bytes) -> Self {
        let extended_pan_id = bytes.get_u64_le();
        let pan_id = bytes.get_u16_le();
        let radio_tx_power = bytes.get_u8();
        let radio_channel = bytes.get_u8();
        let join_method = EmberJoinMethod::decode(bytes.get_u8());
        let nwk_manager_id = bytes.get_u16_le();
        let nwk_update_id = bytes.get_u8();
        let channels = bytes.get_u32_le();
        Self {
            extended_pan_id,
            pan_id,
            radio_tx_power,
            radio_channel,
            join_method,
            nwk_manager_id,
            nwk_update_id,
            channels,
        }
    }
}

impl EmberJoinMethod {
    fn encode(&self) -> u8 {
        match self {
            EmberJoinMethod::UseMacAssociation => 0x00,
            EmberJoinMethod::UseNwkRejoin => 0x01,
            EmberJoinMethod::UseNwkRejoinHaveNwkKey => 0x02,
            EmberJoinMethod::UseConfiguredNwkState => 0x03,
        }
    }

    fn decode(value: u8) -> Self {
        match value {
            0x00 => EmberJoinMethod::UseMacAssociation,
            0x01 => EmberJoinMethod::UseNwkRejoin,
            0x02 => EmberJoinMethod::UseNwkRejoinHaveNwkKey,
            0x03 => EmberJoinMethod::UseConfiguredNwkState,
            _ => panic!("unknown join method"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EzspStatus {
    Success,
    VersionNotSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmberStatus {
    Success,
    FatalError,
    NotJoined,
    SecurityStateNotSet,
}

impl EmberStatus {
    fn encode(&self) -> u8 {
        match self {
            EmberStatus::Success => 0x00,
            EmberStatus::FatalError => 0x01,
            EmberStatus::NotJoined => 0x93,
            EmberStatus::SecurityStateNotSet => 0xA8,
        }
    }

    fn decode(value: u8) -> Self {
        match value {
            0x00 => EmberStatus::Success,
            0x01 => EmberStatus::FatalError,
            0x93 => EmberStatus::NotJoined,
            0xA8 => EmberStatus::SecurityStateNotSet,
            _ => panic!("unknown status: {value:02X}"),
        }
    }
}

impl EmberVersionCommand {
    fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u8(self.desired_protocol_version);
        buffer.freeze()
    }

    fn decode(bytes: &mut Bytes) -> Self {
        let desired_protocol_version = bytes.get_u8();
        Self {
            desired_protocol_version,
        }
    }
}

impl EmberVersionResponse {
    fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u8(self.protocol_version);
        buffer.put_u8(self.stack_type);
        buffer.put_u16_le(self.stack_version);
        buffer.freeze()
    }

    fn decode(bytes: &mut Bytes) -> Self {
        let protocol_version = bytes.get_u8();
        let stack_type = bytes.get_u8();
        let stack_version = bytes.get_u16_le();
        Self {
            protocol_version,
            stack_type,
            stack_version,
        }
    }
}

impl UnknownCommandResponse {
    fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u8(self.status.encode());
        buffer.freeze()
    }

    fn decode(bytes: &mut Bytes) -> Self {
        Self {
            status: EzspStatus::decode(bytes.get_u8()),
        }
    }
}

impl EzspStatus {
    fn encode(&self) -> u8 {
        match self {
            EzspStatus::Success => 0x00,
            EzspStatus::VersionNotSet => 0x30,
        }
    }

    fn decode(value: u8) -> Self {
        match value {
            0x00 => EzspStatus::Success,
            0x30 => EzspStatus::VersionNotSet,
            _ => panic!("unknown status: {value:02X}"),
        }
    }
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
                    Command::NetworkInit(_) => 0x0017,
                    Command::FormNetwork(_) => 0x001E,
                };
                let parameters = match command {
                    Command::Version(command) => command.encode(),
                    Command::NetworkInit(command) => command.encode(),
                    Command::FormNetwork(command) => command.encode(),
                };
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_control_high);
                buffer.put_u16_le(frame_id);
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
                    Response::NetworkInit(_) => 0x0017,
                    Response::FormNetwork(_) => 0x001E,
                    Response::UnknownCommand(_) => 0x0058,
                };
                let parameters = match response {
                    Response::Version(response) => response.encode(),
                    Response::NetworkInit(response) => response.encode(),
                    Response::FormNetwork(response) => response.encode(),
                    Response::UnknownCommand(response) => response.encode(),
                };
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_control_high);
                buffer.put_u16_le(frame_id);
                buffer.put_slice(&parameters);
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
                0x0000 => Command::Version(EmberVersionCommand::decode(&mut parameters)),
                0x0017 => Command::NetworkInit(EmberNetworkInitCommand::decode(&mut parameters)),
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
                0x0000 => Response::Version(EmberVersionResponse::decode(&mut parameters)),
                0x0017 => Response::NetworkInit(EmberNetworkInitResponse::decode(&mut parameters)),
                0x001E => Response::FormNetwork(EmberFormNetworkResponse::decode(&mut parameters)),
                0x0058 => Response::UnknownCommand(UnknownCommandResponse::decode(&mut parameters)),
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
                    Command::NetworkInit(_) => 0x17,
                    Command::FormNetwork(_) => 0x1E,
                };
                let parameters = match command {
                    Command::Version(command) => command.encode(),
                    Command::NetworkInit(command) => command.encode(),
                    Command::FormNetwork(command) => command.encode(),
                };
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_id);
                buffer.put_slice(&parameters);
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
                let frame_id = match response {
                    Response::Version(_) => 0x00,
                    Response::NetworkInit(_) => 0x17,
                    Response::FormNetwork(_) => 0x001E,
                    Response::UnknownCommand(_) => 0x58,
                };
                let parameters = match response {
                    Response::Version(response) => response.encode(),
                    Response::NetworkInit(response) => response.encode(),
                    Response::FormNetwork(response) => response.encode(),
                    Response::UnknownCommand(response) => response.encode(),
                };
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_id);
                buffer.put_slice(&parameters);
            }
        }
        buffer.freeze()
    }

    pub fn decode(bytes: &mut Bytes) -> Self {
        let sequence = bytes.get_u8();
        let frame_control_low = bytes.get_u8();
        let is_command = (frame_control_low & 0b1000_0000) == 0;
        let network_index = (frame_control_low & 0b0110_0000) >> 5;
        let frame_id = bytes.get_u8() as u16;
        let mut parameters = Bytes::from(bytes.to_vec());
        if is_command {
            let sleep_mode = match frame_control_low & 0b0000_0011 {
                0b10 => SleepMode::PowerDown,
                0b01 => SleepMode::DeepSleep,
                0b00 => SleepMode::Idle,
                value => panic!("unknown sleep mode: {value:b}"),
            };
            let command = match frame_id {
                0x0000 => Command::Version(EmberVersionCommand::decode(&mut parameters)),
                0x0017 => Command::NetworkInit(EmberNetworkInitCommand::decode(&mut parameters)),
                _ => panic!("unknown command: {frame_id:02X}"),
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
                0x0000 => Response::Version(EmberVersionResponse::decode(&mut parameters)),
                0x0017 => Response::NetworkInit(EmberNetworkInitResponse::decode(&mut parameters)),
                0x001E => Response::FormNetwork(EmberFormNetworkResponse::decode(&mut parameters)),
                0x0058 => Response::UnknownCommand(UnknownCommandResponse::decode(&mut parameters)),
                _ => panic!("unknown frame id: {frame_id:02X}"),
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
            response: Response::Version(EmberVersionResponse {
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
            response: Response::UnknownCommand(UnknownCommandResponse {
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
            response: Response::NetworkInit(EmberNetworkInitResponse {
                status: EmberStatus::NotJoined,
            }),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decode_form_network_response() {
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
            response: Response::FormNetwork(EmberFormNetworkResponse {
                status: EmberStatus::SecurityStateNotSet,
            }),
        };
        assert_eq!(expected, actual);
    }
}

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
    Version(VersionCommand),
    NetworkInit(NetworkInitCommand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Response {
    Version(VersionResponse),
    NetworkInit(NetworkInitResponse),
    UnknownCommand(UnknownCommandResponse),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionCommand {
    desired_protocol_version: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionResponse {
    protocol_version: u8,
    stack_type: u8,
    stack_version: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownCommandResponse {
    status: EzspStatus,
}

impl NetworkInitCommand {
    pub fn encode(&self) -> Bytes {
        self.bitmask.encode()
    }

    pub fn decode(bytes: &mut Bytes) -> Self {
        Self {
            bitmask: NetworkInitBitmask::decode(bytes),
        }
    }
}

impl NetworkInitBitmask {
    fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u16(match self {
            NetworkInitBitmask::NoOptions => 0x0000,
            NetworkInitBitmask::ParentInfoInToken => 0x0001,
            NetworkInitBitmask::EndDeviceRejoinOnReboot => 0x0002,
        });
        buffer.freeze()
    }

    fn decode(bytes: &mut Bytes) -> Self {
        match bytes.get_u8() {
            0x00 => NetworkInitBitmask::NoOptions,
            0x01 => NetworkInitBitmask::ParentInfoInToken,
            0x02 => NetworkInitBitmask::EndDeviceRejoinOnReboot,
            _ => panic!("unknown bitmask"),
        }
    }
}

impl NetworkInitResponse {
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
pub struct NetworkInitCommand {
    pub bitmask: NetworkInitBitmask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkInitBitmask {
    NoOptions,
    ParentInfoInToken,
    EndDeviceRejoinOnReboot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkInitResponse {
    pub status: EmberStatus,
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
}

impl EmberStatus {
    fn encode(&self) -> u8 {
        match self {
            EmberStatus::Success => 0x00,
            EmberStatus::FatalError => 0x01,
            EmberStatus::NotJoined => 0x93,
        }
    }

    fn decode(value: u8) -> Self {
        match value {
            0x00 => EmberStatus::Success,
            0x01 => EmberStatus::FatalError,
            0x93 => EmberStatus::NotJoined,
            _ => panic!("unknown status: {value:02X}"),
        }
    }
}

impl VersionCommand {
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

impl VersionResponse {
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
                };
                let parameters = match command {
                    Command::Version(command) => command.encode(),
                    Command::NetworkInit(command) => command.encode(),
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
                    Response::UnknownCommand(_) => 0x0058,
                };
                let parameters = match response {
                    Response::Version(response) => response.encode(),
                    Response::NetworkInit(response) => response.encode(),
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
                0x0000 => Command::Version(VersionCommand::decode(&mut parameters)),
                0x0017 => Command::NetworkInit(NetworkInitCommand::decode(&mut parameters)),
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
                0x0000 => Response::Version(VersionResponse::decode(&mut parameters)),
                0x0017 => Response::NetworkInit(NetworkInitResponse::decode(&mut parameters)),
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
                };
                let parameters = match command {
                    Command::Version(command) => command.encode(),
                    Command::NetworkInit(command) => command.encode(),
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
                    Response::UnknownCommand(_) => 0x58,
                };
                let parameters = match response {
                    Response::Version(response) => response.encode(),
                    Response::NetworkInit(response) => response.encode(),
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
                _ => panic!("unknown sleep mode"),
            };
            let command = match frame_id {
                0x0000 => Command::Version(VersionCommand::decode(&mut parameters)),
                0x0017 => Command::NetworkInit(NetworkInitCommand::decode(&mut parameters)),
                _ => panic!("unknown command"),
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
                _ => panic!("unknown callback type"),
            };
            let pending = (frame_control_low >> 2) & 0b1 != 0;
            let truncated = (frame_control_low >> 1) & 0b1 != 0;
            let overflow = frame_control_low & 0b1 != 0;
            let response = match frame_id {
                0x0000 => Response::Version(VersionResponse::decode(&mut parameters)),
                0x0017 => Response::NetworkInit(NetworkInitResponse::decode(&mut parameters)),
                0x0058 => Response::UnknownCommand(UnknownCommandResponse::decode(&mut parameters)),
                _ => panic!("unknown frame id"),
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
    fn test_decode_network_init_response() {
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
}

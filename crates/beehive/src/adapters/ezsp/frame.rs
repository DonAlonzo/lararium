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

enum FrameId {
    Version,
    NetworkInit,
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
        Bytes::new()
    }

    fn decode(bytes: &mut Bytes) -> Self {
        Self {}
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
    //pub status: Status,
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

impl FrameId {
    fn from_u16(value: u16) -> Self {
        match value {
            0x0000 => FrameId::Version,
            0x0017 => FrameId::NetworkInit,
            _ => panic!("unknown frame id"),
        }
    }

    fn to_u16(&self) -> u16 {
        match self {
            FrameId::Version => 0x0000,
            FrameId::NetworkInit => 0x0017,
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
                    Command::Version(_) => FrameId::Version,
                    Command::NetworkInit(_) => FrameId::NetworkInit,
                };
                let parameters = match command {
                    Command::Version(command) => command.encode(),
                    Command::NetworkInit(command) => command.encode(),
                };
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_control_high);
                buffer.put_u16(frame_id.to_u16());
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
                    Response::Version(_) => FrameId::Version,
                    Response::NetworkInit(_) => FrameId::NetworkInit,
                };
                let parameters = match response {
                    Response::Version(response) => response.encode(),
                    Response::NetworkInit(response) => response.encode(),
                };
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_control_high);
                buffer.put_u16(frame_id.to_u16());
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
        if frame_control_high & 0b0000_0001 != 0 {
            panic!("unknown frame format version");
        }
        let frame_id = FrameId::from_u16(bytes.get_u8() as u16);
        let mut parameters = Bytes::from(bytes.to_vec());
        if is_command {
            let sleep_mode = match frame_control_low & 0b0000_0011 {
                0b10 => SleepMode::PowerDown,
                0b01 => SleepMode::DeepSleep,
                0b00 => SleepMode::Idle,
                _ => panic!("unknown sleep mode"),
            };
            let command = match frame_id {
                FrameId::Version => Command::Version(VersionCommand::decode(&mut parameters)),
                FrameId::NetworkInit => {
                    Command::NetworkInit(NetworkInitCommand::decode(&mut parameters))
                }
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
                FrameId::Version => Response::Version(VersionResponse::decode(&mut parameters)),
                FrameId::NetworkInit => {
                    Response::NetworkInit(NetworkInitResponse::decode(&mut parameters))
                }
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
                    Command::Version(_) => FrameId::Version,
                    Command::NetworkInit(_) => FrameId::NetworkInit,
                };
                let parameters = match command {
                    Command::Version(command) => command.encode(),
                    Command::NetworkInit(command) => command.encode(),
                };
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_id.to_u16() as u8);
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
                    Response::Version(_) => FrameId::Version,
                    Response::NetworkInit(_) => FrameId::NetworkInit,
                };
                let parameters = match response {
                    Response::Version(response) => response.encode(),
                    Response::NetworkInit(response) => response.encode(),
                };
                buffer.put_u8(*sequence);
                buffer.put_u8(frame_control_low);
                buffer.put_u8(frame_id.to_u16() as u8);
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
        let frame_id = FrameId::from_u16(bytes.get_u8() as u16);
        let mut parameters = Bytes::from(bytes.to_vec());
        if is_command {
            let sleep_mode = match frame_control_low & 0b0000_0011 {
                0b10 => SleepMode::PowerDown,
                0b01 => SleepMode::DeepSleep,
                0b00 => SleepMode::Idle,
                _ => panic!("unknown sleep mode"),
            };
            let command = match frame_id {
                FrameId::Version => Command::Version(VersionCommand::decode(&mut parameters)),
                FrameId::NetworkInit => {
                    Command::NetworkInit(NetworkInitCommand::decode(&mut parameters))
                }
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
                FrameId::Version => Response::Version(VersionResponse::decode(&mut parameters)),
                FrameId::NetworkInit => {
                    Response::NetworkInit(NetworkInitResponse::decode(&mut parameters))
                }
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
    fn test_decode_version() {
        let mut bytes = Bytes::from_static(&[0x00, 0x80, 0x00, 0x0D, 0x02, 0x30, 0x74]);
        let actual = FrameVersion0::decode(&mut bytes);
        let expected = FrameVersion0::Response {
            sequence: 0x00,
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
}

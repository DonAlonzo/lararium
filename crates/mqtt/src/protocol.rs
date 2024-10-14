use bytes::{Buf, BufMut};

#[derive(Debug)]
pub enum Error {
    Invalid,
    Incomplete,
    UnsupportedProtocol,
    UnsupportedProtocolVersion,
}

impl std::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter,
    ) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

#[derive(Debug, PartialEq)]
pub enum ControlPacket {
    Connect {},
    Connack {
        reason_code: ConnectReasonCode,
    },
    Publish {
        topic_name: String,
        payload: Vec<u8>,
    },
    Puback {},
}

impl ControlPacket {
    pub fn decode(input: &[u8]) -> Result<(Self, usize), Error> {
        let mut buf = &input[..];

        // Fixed header
        let packet_type_and_flags = buf.get_u8();
        let packet_type = packet_type_and_flags >> 4;
        let flags = packet_type_and_flags & 0x0F;
        let remaining_length = buf.get_u8();

        if buf.remaining() < remaining_length as usize {
            return Err(Error::Incomplete);
        }

        let packet = match packet_type {
            // 3.1.2 CONNECT Variable Header
            0x01 => {
                // 3.1.2.1 Protocol Name
                let protocol_name_length = buf.get_u16();
                let protocol_name = &buf[..protocol_name_length as usize];
                if protocol_name != b"MQTT" {
                    return Err(Error::UnsupportedProtocol);
                }
                buf.advance(protocol_name_length as usize);

                // 3.1.2.2 Protocol Version
                let protocol_version = match buf.get_u8() {
                    0x04 => Protocol::V3_1_1,
                    0x05 => Protocol::V5_0,
                    _ => return Err(Error::UnsupportedProtocolVersion),
                };

                // 3.1.2.3 Connect Flags
                let connect_flags = buf.get_u8();
                let clean_start = (connect_flags & 0b00000010) != 0;
                let will_flag = (connect_flags & 0b00000100) != 0;
                let will_qos = match (connect_flags & 0b00011000) >> 3 {
                    0 => QoS::AtMostOnce,
                    1 if will_flag => QoS::AtLeastOnce,
                    2 if will_flag => QoS::ExactlyOnce,
                    _ => return Err(Error::Invalid),
                };
                let will_retain = match (connect_flags & 0b00100000) >> 5 {
                    0 => false,
                    1 if will_flag => true,
                    _ => return Err(Error::Invalid),
                };
                let password_flag = (connect_flags & 0b01000000) != 0;
                let username_flag = (connect_flags & 0b10000000) != 0;

                // 3.1.2.10 Keep Alive
                let keep_alive = buf.get_u16();

                // 3.1.2.11 CONNECT Properties
                let properties_length = {
                    let mut variable_length: u32 = buf.get_u8() as u32;
                    if variable_length == 0xFF {
                        variable_length = (variable_length << 8) & buf.get_u8() as u32;
                    }
                    if variable_length == 0xFFFF {
                        variable_length = (variable_length << 8) & buf.get_u8() as u32;
                    }
                    if variable_length == 0xFFFFFF {
                        variable_length = (variable_length << 8) & buf.get_u8() as u32;
                    }
                    variable_length
                };

                // TODO
                buf.get_u8();

                ControlPacket::Connect {}
            }
            // 3.2.2 CONNACK Variable Header
            0x04 => {
                //
                ControlPacket::Connack {
                    reason_code: ConnectReasonCode::Success,
                }
            }
            // 3.3.2 PUBLISH Variable Header
            0x03 => {
                let retain_flag = (flags & 0b00000001) != 0;
                let qos = (flags & 0b00000110) >> 5;
                let dup_flag = (flags & 0b00001000) != 0;

                // 3.3.2.1 Topic Name
                let topic_name_length = buf.get_u16();
                let topic_name = &buf[..topic_name_length as usize];
                let Ok(topic_name) = std::str::from_utf8(topic_name) else {
                    return Err(Error::Invalid);
                };
                buf.advance(topic_name_length as usize);

                // 3.3.3 PUBLISH Payload
                let payload = &buf[..(remaining_length as usize - buf.remaining())];
                buf.advance(payload.len());

                ControlPacket::Publish {
                    topic_name: topic_name.into(),
                    payload: payload.to_vec(),
                }
            }
            // 3.4.2 PUBACK Variable Header
            0x04 => {
                //
                ControlPacket::Puback {}
            }
            _ => return Err(Error::Invalid),
        };
        Ok((packet, buf.remaining()))
    }

    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        match self {
            ControlPacket::Connect {} => {
                buffer.push(0x10);
                buffer.push(0x0c);
                buffer.push(0x00);
                buffer.push(0x04);
                buffer.extend_from_slice(b"MQTT");
                buffer.push(0x04);
                buffer.push(0x02);
                buffer.push(0x00);
                buffer.push(0x00);
                buffer.push(0x00);
                buffer.push(0x00);
            }
            ControlPacket::Connack { reason_code } => {
                buffer.push(0x20);
                buffer.push(0x02);
                buffer.push(0x00);
                buffer.push(0x00);
            }
            ControlPacket::Publish {
                topic_name,
                payload,
            } => {
                buffer.push(0x30);
                buffer.push(0x18);
                buffer.put_u16(topic_name.len() as u16);
                buffer.extend_from_slice(topic_name.as_bytes());
                buffer.extend_from_slice(payload.as_slice());
            }
            ControlPacket::Puback {} => {
                buffer.push(0x40);
                buffer.push(0x00);
            }
        }
        Ok(buffer)
    }
}

#[derive(Debug, PartialEq)]
pub enum Protocol {
    V3_1_1,
    V5_0,
}

#[derive(Debug, PartialEq)]
pub enum ConnectReasonCode {
    Success,
    UnspecifiedError,
    MalformedPacket,
    ProtocolError,
    ImplementationSpecificError,
    UnsupportedProtocolVersion,
    ClientIdentifierNotValid,
    BadUserNameOrPassword,
    NotAuthorized,
    ServerUnavailable,
    ServerBusy,
    Banned,
    BadAuthenticationMethod,
    TopicNameInvalid,
    PacketTooLarge,
    QuotaExceeded,
    PayloadFormatInvalid,
    RetainNotSupported,
    QoSNotSupported,
    UseAnotherServer,
    ServerMoved,
    ConnectionRateExceeded,
}

#[derive(Debug, PartialEq)]
pub enum PubackReasonCode {
    Success,
    NoMatchingSubscribers,
    UnspecifiedError,
    ImplementationSpecificError,
    NotAuthorized,
    TopicNameInvalid,
    PacketIdentifierInUse,
    QuotaExceeded,
    PayloadFormatInvalid,
}

#[derive(Debug, PartialEq)]
pub struct ConnectFlags {
    //will_retain: bool,
    //will_qos: QoS,
    //will_flag: bool,
    clean_start: bool,
}

#[derive(Debug, PartialEq)]
pub enum QoS {
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

#[derive(Debug, PartialEq)]
pub struct ConnectAcknowledgeFlags {
    session_present: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_connect() {
        let packet = [
            0x10, 0x0c, 0x00, 0x04, b'M', b'Q', b'T', b'T', 0x04, 0x02, 0x00, 0x00, 0x00, 0x00,
        ];
        let (packet, read_bytes) = ControlPacket::decode(&packet).unwrap();
        assert_eq!(packet, ControlPacket::Connect {});
        assert_eq!(read_bytes, 0);
    }

    #[test]
    fn test_encode_connack() {
        let packet = ControlPacket::Connack {
            reason_code: ConnectReasonCode::Success,
        };
        let actual = packet.encode().unwrap();
        let expected = [0x20, 0x02, 0x00, 0x00];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decode_publish() {
        let packet = [
            0x30, 0x18, 0x00, 0x0a, 0x74, 0x65, 0x73, 0x74, 0x2f, 0x74, 0x6f, 0x70, 0x69, 0x63,
            0x74, 0x65, 0x73, 0x74, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65,
        ];
        let (packet, read_bytes) = ControlPacket::decode(&packet).unwrap();
        assert_eq!(
            packet,
            ControlPacket::Publish {
                topic_name: "test/topic".into(),
                payload: b"test message".to_vec(),
            }
        );
        assert_eq!(read_bytes, 0);
    }

    #[test]
    fn test_encode_puback() {
        let packet = ControlPacket::Puback {};
        let actual = packet.encode().unwrap();
        let expected = [0x40, 0x00];
        assert_eq!(actual, expected);
    }
}

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
    Connect {
        clean_start: bool,
    },
    Connack {
        reason_code: ConnectReasonCode,
    },
    Publish {
        topic_name: String,
        payload: Vec<u8>,
    },
    Puback {},
    Subscribe {
        packet_identifier: u16,
        topic_name: String,
    },
    Suback {
        packet_identifier: u16,
        reason_codes: Vec<SubscribeReasonCode>,
    },
    Pingreq,
    Pingresp,
    Disconnect {
        reason_code: DisconnectReasonCode,
    },
}

pub enum PacketType {
    Connect,
    Connack,
    Publish,
    Puback,
    Pubrec,
    Pubrel,
    Pubcomp,
    Subscribe,
    Suback,
    Unsubscribe,
    Unsuback,
    Pingreq,
    Pingresp,
    Disconnect,
    Auth,
}

impl ControlPacket {
    pub fn decode(input: &[u8]) -> Result<(Self, usize), Error> {
        let mut buf = &input[..];

        // Fixed header
        let packet_type_and_flags = buf.get_u8();
        let packet_type = packet_type_and_flags >> 4;
        let packet_type = match packet_type {
            0b0001 => PacketType::Connect,
            0b0010 => PacketType::Connack,
            0b0011 => PacketType::Publish,
            0b0100 => PacketType::Puback,
            0b0101 => PacketType::Pubrec,
            0b0110 => PacketType::Pubrel,
            0b0111 => PacketType::Pubcomp,
            0b1000 => PacketType::Subscribe,
            0b1001 => PacketType::Suback,
            0b1010 => PacketType::Unsubscribe,
            0b1011 => PacketType::Unsuback,
            0b1100 => PacketType::Pingreq,
            0b1101 => PacketType::Pingresp,
            0b1110 => PacketType::Disconnect,
            0b1111 => PacketType::Auth,
            _ => return Err(Error::Invalid),
        };
        let flags = packet_type_and_flags & 0x0F;
        let remaining_length = buf.get_u8();

        if buf.remaining() < remaining_length as usize {
            return Err(Error::Incomplete);
        }

        let packet = match packet_type {
            // 3.1.2 CONNECT Variable Header
            PacketType::Connect => {
                // 3.1.2.1 Protocol Name
                let protocol_name_length = buf.get_u16();
                let protocol_name = &buf[..protocol_name_length as usize];
                if protocol_name != b"MQTT" {
                    return Err(Error::UnsupportedProtocol);
                }
                buf.advance(protocol_name_length as usize);

                // 3.1.2.2 Protocol Version
                match buf.get_u8() {
                    0x04 => Protocol::V3_1_1,
                    0x05 => Protocol::V5_0,
                    _ => return Err(Error::UnsupportedProtocolVersion),
                };

                // 3.1.2.3 Connect Flags
                let connect_flags = buf.get_u8();
                let clean_start = (connect_flags & 0b00000010) != 0;
                let will_flag = (connect_flags & 0b00000100) != 0;
                let will_qos = match (connect_flags & 0b00011000) >> 3 {
                    0b00 => QoS::AtMostOnce,
                    0b01 if will_flag => QoS::AtLeastOnce,
                    0b10 if will_flag => QoS::ExactlyOnce,
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
                let Ok(properties_length) = buf.get_variable_length() else {
                    return Err(Error::Invalid);
                };

                // TODO
                buf.get_u8();

                ControlPacket::Connect {
                    clean_start,
                    //will: Will {
                    //    retain: will_retain,
                    //    qos: will_qos,
                    //},
                }
            }
            // 3.2.2 CONNACK Variable Header
            PacketType::Connack => {
                // 3.2.2.1 Connect Acknowledge Flags
                let connect_acknowledge_flags = buf.get_u8();
                let session_present = (connect_acknowledge_flags & 0b00000001) != 0;

                // 3.2.2.2 Connect Reason Code
                let reason_code = match buf.get_u8() {
                    0x00 => ConnectReasonCode::Success,
                    0x80 => ConnectReasonCode::UnspecifiedError,
                    0x81 => ConnectReasonCode::MalformedPacket,
                    0x82 => ConnectReasonCode::ProtocolError,
                    0x83 => ConnectReasonCode::ImplementationSpecificError,
                    0x84 => ConnectReasonCode::UnsupportedProtocolVersion,
                    0x85 => ConnectReasonCode::ClientIdentifierNotValid,
                    0x86 => ConnectReasonCode::BadUserNameOrPassword,
                    0x87 => ConnectReasonCode::NotAuthorized,
                    0x88 => ConnectReasonCode::ServerUnavailable,
                    0x89 => ConnectReasonCode::ServerBusy,
                    0x8A => ConnectReasonCode::Banned,
                    0x8B => ConnectReasonCode::BadAuthenticationMethod,
                    0x8C => ConnectReasonCode::TopicNameInvalid,
                    0x8F => ConnectReasonCode::PacketTooLarge,
                    0x97 => ConnectReasonCode::QuotaExceeded,
                    0x99 => ConnectReasonCode::PayloadFormatInvalid,
                    0x9A => ConnectReasonCode::RetainNotSupported,
                    0x9B => ConnectReasonCode::QoSNotSupported,
                    0x9C => ConnectReasonCode::UseAnotherServer,
                    0x9D => ConnectReasonCode::ServerMoved,
                    0x9F => ConnectReasonCode::ConnectionRateExceeded,
                    _ => return Err(Error::Invalid),
                };

                ControlPacket::Connack {
                    reason_code: ConnectReasonCode::Success,
                }
            }
            // 3.3.2 PUBLISH Variable Header
            PacketType::Publish => {
                let retain_flag = (flags & 0b00000001) != 0;
                let qos = match (flags & 0b00000110) >> 1 {
                    0b00 => QoS::AtMostOnce,
                    0b01 => QoS::AtLeastOnce,
                    0b10 => QoS::ExactlyOnce,
                    _ => return Err(Error::Invalid),
                };
                let dup_flag = (flags & 0b00001000) != 0;

                // 3.3.2.1 Topic Name
                let topic_name_length = buf.get_u16();
                let topic_name = &buf[..topic_name_length as usize];
                let Ok(topic_name) = std::str::from_utf8(topic_name) else {
                    return Err(Error::Invalid);
                };
                buf.advance(topic_name_length as usize);

                // 3.3.3 PUBLISH Payload
                let payload =
                    &buf[..(remaining_length as usize - (input.len() - buf.remaining() - 2))];
                buf.advance(payload.len());

                ControlPacket::Publish {
                    topic_name: topic_name.into(),
                    payload: payload.to_vec(),
                }
            }
            // 3.4.2 PUBACK Variable Header
            PacketType::Puback => {
                //
                ControlPacket::Puback {}
            }
            PacketType::Pubrec => todo!(),
            PacketType::Pubrel => todo!(),
            PacketType::Pubcomp => todo!(),
            PacketType::Subscribe => {
                let packet_identifier = buf.get_u16();

                // 3.8.3 SUBSCRIBE Payload
                let topic_name_length = buf.get_u16();
                let topic_name = &buf[..topic_name_length as usize];
                let Ok(topic_name) = std::str::from_utf8(topic_name) else {
                    return Err(Error::Invalid);
                };
                buf.advance(topic_name_length as usize);
                let subscription_options = buf.get_u8();

                ControlPacket::Subscribe {
                    packet_identifier,
                    topic_name: topic_name.into(),
                }
            }
            PacketType::Suback => {
                let packet_identifier = buf.get_u16();
                let qos = buf.get_u8();

                ControlPacket::Suback {
                    packet_identifier,
                    reason_codes: vec![SubscribeReasonCode::GrantedQoS0],
                }
            }
            PacketType::Unsubscribe => todo!(),
            PacketType::Unsuback => todo!(),
            PacketType::Pingreq => {
                //
                ControlPacket::Pingreq
            }
            PacketType::Pingresp => {
                //
                ControlPacket::Pingresp
            }
            PacketType::Disconnect => {
                //
                let reason_code = if remaining_length > 0 {
                    match buf.get_u8() {
                        0x00 => DisconnectReasonCode::NormalDisconnection,
                        0x04 => DisconnectReasonCode::DisconnectWithWillMessage,
                        0x80 => DisconnectReasonCode::UnspecifiedError,
                        0x81 => DisconnectReasonCode::MalformedPacket,
                        0x82 => DisconnectReasonCode::ProtocolError,
                        0x83 => DisconnectReasonCode::ImplementationSpecificError,
                        0x87 => DisconnectReasonCode::NotAuthorized,
                        0x89 => DisconnectReasonCode::ServerBusy,
                        0x8B => DisconnectReasonCode::ServerShuttingDown,
                        0x8D => DisconnectReasonCode::KeepAliveTimeout,
                        0x8E => DisconnectReasonCode::SessionTakenOver,
                        0x8F => DisconnectReasonCode::TopicFilterInvalid,
                        0x90 => DisconnectReasonCode::TopicNameInvalid,
                        0x93 => DisconnectReasonCode::ReceiveMaximumExceeded,
                        0x94 => DisconnectReasonCode::TopicAliasInvalid,
                        0x95 => DisconnectReasonCode::PacketTooLarge,
                        0x96 => DisconnectReasonCode::MessageRateTooHigh,
                        0x97 => DisconnectReasonCode::QuotaExceeded,
                        0x98 => DisconnectReasonCode::AdministrativeAction,
                        0x99 => DisconnectReasonCode::PayloadFormatInvalid,
                        0x9A => DisconnectReasonCode::RetainNotSupported,
                        0x9B => DisconnectReasonCode::QoSNotSupported,
                        0x9C => DisconnectReasonCode::UseAnotherServer,
                        0x9D => DisconnectReasonCode::ServerMoved,
                        0x9E => DisconnectReasonCode::SharedSubscriptionsNotSupported,
                        0x9F => DisconnectReasonCode::ConnectionRateExceeded,
                        0xA0 => DisconnectReasonCode::MaximumConnectTime,
                        0xA1 => DisconnectReasonCode::SubscriptionIdentifiersNotSupported,
                        0xA2 => DisconnectReasonCode::WildcardSubscriptionsNotSupported,
                        _ => return Err(Error::Invalid),
                    }
                } else {
                    DisconnectReasonCode::NormalDisconnection
                };

                ControlPacket::Disconnect {
                    reason_code: DisconnectReasonCode::NormalDisconnection,
                }
            }
            PacketType::Auth => todo!(),
        };
        Ok((packet, buf.remaining()))
    }

    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        match self {
            ControlPacket::Connect { clean_start } => {
                buffer.put_u8(0x10);
                buffer.put_u8(0x0c);
                buffer.put_u8(0x00);
                buffer.put_u8(0x04);
                buffer.extend_from_slice(b"MQTT");
                buffer.put_u8(0x04);
                buffer.put_u8(0x02);
                buffer.put_u8(0x00);
                buffer.put_u8(0x00);
                buffer.put_u8(0x00);
                buffer.put_u8(0x00);
            }
            ControlPacket::Connack { reason_code } => {
                buffer.put_u8(0x20);
                buffer.put_u8(0x02);
                buffer.put_u8(0x00);
                buffer.put_u8(0x00);
            }
            ControlPacket::Publish {
                topic_name,
                payload,
            } => {
                buffer.put_u8(0x30);
                buffer.put_variable_length((2 + topic_name.len() + payload.len()) as u32);
                buffer.put_u16(topic_name.len() as u16);
                buffer.extend_from_slice(topic_name.as_bytes());
                buffer.extend_from_slice(payload.as_slice());
            }
            ControlPacket::Puback {} => {
                buffer.put_u8(0x40);
                buffer.put_u8(0x00);
            }
            ControlPacket::Subscribe {
                packet_identifier,
                topic_name,
            } => {
                buffer.put_u8(0x82); // control packet type and flags
                buffer.put_variable_length((4 + topic_name.len() + 1) as u32); // remaining length
                buffer.put_u16(*packet_identifier);
                buffer.put_u16(topic_name.len() as u16);
                buffer.extend_from_slice(topic_name.as_bytes());
                buffer.put_u8(0x00); // subscription options
            }
            ControlPacket::Suback {
                packet_identifier,
                reason_codes,
            } => {
                buffer.put_u8(0x90); // control packet type and flags
                buffer.put_variable_length((2 + reason_codes.len()) as u32); // remaining length
                buffer.put_u16(*packet_identifier);
                for reason_code in reason_codes {
                    buffer.put_u8(match reason_code {
                        SubscribeReasonCode::GrantedQoS0 => 0x00,
                        SubscribeReasonCode::GrantedQoS1 => 0x01,
                        SubscribeReasonCode::GrantedQoS2 => 0x02,
                        SubscribeReasonCode::UnspecifiedError => 0x80,
                        SubscribeReasonCode::ImplementationSpecificError => 0x83,
                        SubscribeReasonCode::NotAuthorized => 0x87,
                        SubscribeReasonCode::TopicFilterInvalid => 0x8F,
                        SubscribeReasonCode::PacketIdentifierInUse => 0x91,
                        SubscribeReasonCode::QuotaExceeded => 0x97,
                        SubscribeReasonCode::SharedSubscriptionsNotSupported => 0x9E,
                        SubscribeReasonCode::SubscriptionIdentifiersNotSupported => 0xA1,
                        SubscribeReasonCode::WildcardSubscriptionsNotSupported => 0xA2,
                    });
                }
            }
            ControlPacket::Pingreq => {
                buffer.put_u8(0xC0);
                buffer.put_u8(0x00);
            }
            ControlPacket::Pingresp => {
                buffer.put_u8(0xD0);
                buffer.put_u8(0x00);
            }
            ControlPacket::Disconnect { reason_code } => {
                buffer.put_u8(0xE0);
                buffer.put_u8(match reason_code {
                    DisconnectReasonCode::NormalDisconnection => 0x00,
                    DisconnectReasonCode::DisconnectWithWillMessage => 0x04,
                    DisconnectReasonCode::UnspecifiedError => 0x80,
                    DisconnectReasonCode::MalformedPacket => 0x81,
                    DisconnectReasonCode::ProtocolError => 0x82,
                    DisconnectReasonCode::ImplementationSpecificError => 0x83,
                    DisconnectReasonCode::NotAuthorized => 0x87,
                    DisconnectReasonCode::ServerBusy => 0x89,
                    DisconnectReasonCode::ServerShuttingDown => 0x8B,
                    DisconnectReasonCode::KeepAliveTimeout => 0x8D,
                    DisconnectReasonCode::SessionTakenOver => 0x8E,
                    DisconnectReasonCode::TopicFilterInvalid => 0x8F,
                    DisconnectReasonCode::TopicNameInvalid => 0x90,
                    DisconnectReasonCode::ReceiveMaximumExceeded => 0x93,
                    DisconnectReasonCode::TopicAliasInvalid => 0x94,
                    DisconnectReasonCode::PacketTooLarge => 0x95,
                    DisconnectReasonCode::MessageRateTooHigh => 0x96,
                    DisconnectReasonCode::QuotaExceeded => 0x97,
                    DisconnectReasonCode::AdministrativeAction => 0x98,
                    DisconnectReasonCode::PayloadFormatInvalid => 0x99,
                    DisconnectReasonCode::RetainNotSupported => 0x9A,
                    DisconnectReasonCode::QoSNotSupported => 0x9B,
                    DisconnectReasonCode::UseAnotherServer => 0x9C,
                    DisconnectReasonCode::ServerMoved => 0x9D,
                    DisconnectReasonCode::SharedSubscriptionsNotSupported => 0x9E,
                    DisconnectReasonCode::ConnectionRateExceeded => 0x9F,
                    DisconnectReasonCode::MaximumConnectTime => 0xA0,
                    DisconnectReasonCode::SubscriptionIdentifiersNotSupported => 0xA1,
                    DisconnectReasonCode::WildcardSubscriptionsNotSupported => 0xA2,
                });
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
pub enum SubscribeReasonCode {
    GrantedQoS0,
    GrantedQoS1,
    GrantedQoS2,
    UnspecifiedError,
    ImplementationSpecificError,
    NotAuthorized,
    TopicFilterInvalid,
    PacketIdentifierInUse,
    QuotaExceeded,
    SharedSubscriptionsNotSupported,
    SubscriptionIdentifiersNotSupported,
    WildcardSubscriptionsNotSupported,
}

#[derive(Debug, PartialEq)]
pub enum DisconnectReasonCode {
    NormalDisconnection,
    DisconnectWithWillMessage,
    UnspecifiedError,
    MalformedPacket,
    ProtocolError,
    ImplementationSpecificError,
    NotAuthorized,
    ServerBusy,
    ServerShuttingDown,
    KeepAliveTimeout,
    SessionTakenOver,
    TopicFilterInvalid,
    TopicNameInvalid,
    ReceiveMaximumExceeded,
    TopicAliasInvalid,
    PacketTooLarge,
    MessageRateTooHigh,
    QuotaExceeded,
    AdministrativeAction,
    PayloadFormatInvalid,
    RetainNotSupported,
    QoSNotSupported,
    UseAnotherServer,
    ServerMoved,
    SharedSubscriptionsNotSupported,
    ConnectionRateExceeded,
    MaximumConnectTime,
    SubscriptionIdentifiersNotSupported,
    WildcardSubscriptionsNotSupported,
}

#[derive(Debug, PartialEq)]
pub struct ConnectFlags {
    will: Will,
    clean_start: bool,
}

#[derive(Debug, PartialEq)]
pub struct Will {
    retain: bool,
    qos: QoS,
}

#[derive(Debug, PartialEq)]
pub enum QoS {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

#[derive(Debug, PartialEq)]
pub struct ConnectAcknowledgeFlags {
    session_present: bool,
}

pub trait BufExt {
    fn get_variable_length(&mut self) -> Result<u32, ()>;
}

pub trait BufMutExt {
    fn put_variable_length(
        &mut self,
        value: u32,
    );
}

impl<T: Buf> BufExt for T {
    fn get_variable_length(&mut self) -> Result<u32, ()> {
        let mut multiplier: u32 = 1;
        let mut value: u32 = 0;
        let mut encoded_byte;
        loop {
            encoded_byte = self.get_u8();
            value += (encoded_byte as u32 & 127) * multiplier;
            if (encoded_byte & 128) == 0 {
                break;
            }
            multiplier *= 128;
            if multiplier > 128 * 128 * 128 {
                return Err(());
            }
        }
        Ok(value)
    }
}

impl<T: BufMut> BufMutExt for T {
    fn put_variable_length(
        &mut self,
        mut value: u32,
    ) {
        loop {
            let mut encoded_byte = (value % 128) as u8;
            value /= 128;
            if value > 0 {
                encoded_byte |= 128;
            }
            self.put_u8(encoded_byte);
            if value == 0 {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_connect() {
        let packet = ControlPacket::Connect { clean_start: true };
        let actual = packet.encode().unwrap();
        let expected = [
            0x10, 0x0c, 0x00, 0x04, b'M', b'Q', b'T', b'T', 0x04, 0x02, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decode_connect() {
        let packet = [
            0x10, 0x0c, 0x00, 0x04, b'M', b'Q', b'T', b'T', 0x04, 0x02, 0x00, 0x00, 0x00, 0x00,
        ];
        let (actual, remaining_bytes) = ControlPacket::decode(&packet).unwrap();
        let expected = ControlPacket::Connect { clean_start: true };
        assert_eq!(actual, expected);
        assert_eq!(remaining_bytes, 0);
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
    fn test_decode_connack() {
        let packet = [0x20, 0x02, 0x00, 0x00];
        let (actual, remaining_bytes) = ControlPacket::decode(&packet).unwrap();
        let expected = ControlPacket::Connack {
            reason_code: ConnectReasonCode::Success,
        };
        assert_eq!(actual, expected);
        assert_eq!(remaining_bytes, 0);
    }

    #[test]
    fn test_encode_publish_1() {
        let packet = ControlPacket::Publish {
            topic_name: "test/topic".into(),
            payload: b"test message".to_vec(),
        };
        let actual = packet.encode().unwrap();
        let expected = [
            0x30, 0x18, 0x00, 0x0a, 0x74, 0x65, 0x73, 0x74, 0x2f, 0x74, 0x6f, 0x70, 0x69, 0x63,
            0x74, 0x65, 0x73, 0x74, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65,
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_publish_2() {
        let packet = ControlPacket::Publish {
            topic_name: "abc/def/ghi/jkl/mno".into(),
            payload: b"all your base are belong to us".to_vec(),
        };
        let actual = packet.encode().unwrap();
        let expected = [
            0x30, 0x33, 0x00, 0x13, 0x61, 0x62, 0x63, 0x2f, 0x64, 0x65, 0x66, 0x2f, 0x67, 0x68,
            0x69, 0x2f, 0x6a, 0x6b, 0x6c, 0x2f, 0x6d, 0x6e, 0x6f, 0x61, 0x6c, 0x6c, 0x20, 0x79,
            0x6f, 0x75, 0x72, 0x20, 0x62, 0x61, 0x73, 0x65, 0x20, 0x61, 0x72, 0x65, 0x20, 0x62,
            0x65, 0x6c, 0x6f, 0x6e, 0x67, 0x20, 0x74, 0x6f, 0x20, 0x75, 0x73,
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decode_publish_1() {
        let packet = [
            0x30, 0x18, 0x00, 0x0a, 0x74, 0x65, 0x73, 0x74, 0x2f, 0x74, 0x6f, 0x70, 0x69, 0x63,
            0x74, 0x65, 0x73, 0x74, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65,
        ];
        let (actual, remaining_bytes) = ControlPacket::decode(&packet).unwrap();
        let expected = ControlPacket::Publish {
            topic_name: "test/topic".into(),
            payload: b"test message".to_vec(),
        };
        assert_eq!(actual, expected);
        assert_eq!(remaining_bytes, 0);
    }

    #[test]
    fn test_decode_publish_2() {
        let packet = [
            0x30, 0x33, 0x00, 0x13, 0x61, 0x62, 0x63, 0x2f, 0x64, 0x65, 0x66, 0x2f, 0x67, 0x68,
            0x69, 0x2f, 0x6a, 0x6b, 0x6c, 0x2f, 0x6d, 0x6e, 0x6f, 0x61, 0x6c, 0x6c, 0x20, 0x79,
            0x6f, 0x75, 0x72, 0x20, 0x62, 0x61, 0x73, 0x65, 0x20, 0x61, 0x72, 0x65, 0x20, 0x62,
            0x65, 0x6c, 0x6f, 0x6e, 0x67, 0x20, 0x74, 0x6f, 0x20, 0x75, 0x73,
        ];
        let (actual, remaining_bytes) = ControlPacket::decode(&packet).unwrap();
        let expected = ControlPacket::Publish {
            topic_name: "abc/def/ghi/jkl/mno".into(),
            payload: b"all your base are belong to us".to_vec(),
        };
        assert_eq!(actual, expected);
        assert_eq!(remaining_bytes, 0);
    }

    #[test]
    fn test_encode_puback() {
        let packet = ControlPacket::Puback {};
        let actual = packet.encode().unwrap();
        let expected = [0x40, 0x00];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decode_puback() {
        let packet = [0x40, 0x00];
        let (actual, remaining_bytes) = ControlPacket::decode(&packet).unwrap();
        let expected = ControlPacket::Puback {};
        assert_eq!(actual, expected);
        assert_eq!(remaining_bytes, 0);
    }

    #[test]
    fn test_encode_subscribe_1() {
        let packet = ControlPacket::Subscribe {
            packet_identifier: 4,
            topic_name: "lararium/station".into(),
        };
        let actual = packet.encode().unwrap();
        let expected = [
            0x82, 0x15, 0x00, 0x04, 0x00, 0x10, 0x6c, 0x61, 0x72, 0x61, 0x72, 0x69, 0x75, 0x6d,
            0x2f, 0x73, 0x74, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x00,
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_encode_subscribe_2() {
        let packet = ControlPacket::Subscribe {
            packet_identifier: 3,
            topic_name: "lararium/beehive".into(),
        };
        let actual = packet.encode().unwrap();
        let expected = [
            0x82, 0x15, 0x00, 0x03, 0x00, 0x10, 0x6c, 0x61, 0x72, 0x61, 0x72, 0x69, 0x75, 0x6d,
            0x2f, 0x62, 0x65, 0x65, 0x68, 0x69, 0x76, 0x65, 0x00,
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decode_subscribe_1() {
        let packet = [
            0x82, 0x15, 0x00, 0x04, 0x00, 0x10, 0x6c, 0x61, 0x72, 0x61, 0x72, 0x69, 0x75, 0x6d,
            0x2f, 0x73, 0x74, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x00,
        ];
        let (actual, remaining_bytes) = ControlPacket::decode(&packet).unwrap();
        let expected = ControlPacket::Subscribe {
            packet_identifier: 4,
            topic_name: "lararium/station".into(),
        };
        assert_eq!(actual, expected);
        assert_eq!(remaining_bytes, 0);
    }

    #[test]
    fn test_decode_subscribe_2() {
        let packet = [
            0x82, 0x15, 0x00, 0x03, 0x00, 0x10, 0x6c, 0x61, 0x72, 0x61, 0x72, 0x69, 0x75, 0x6d,
            0x2f, 0x62, 0x65, 0x65, 0x68, 0x69, 0x76, 0x65, 0x00,
        ];
        let (actual, remaining_bytes) = ControlPacket::decode(&packet).unwrap();
        let expected = ControlPacket::Subscribe {
            packet_identifier: 3,
            topic_name: "lararium/beehive".into(),
        };
        assert_eq!(actual, expected);
        assert_eq!(remaining_bytes, 0);
    }

    #[test]
    fn test_encode_suback() {
        let packet = ControlPacket::Suback {
            packet_identifier: 4,
            reason_codes: vec![SubscribeReasonCode::GrantedQoS0],
        };
        let actual = packet.encode().unwrap();
        let expected = [0x90, 0x03, 0x00, 0x04, 0x00];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decode_suback() {
        let packet = [0x90, 0x03, 0x00, 0x04, 0x00];
        let (actual, remaining_bytes) = ControlPacket::decode(&packet).unwrap();
        let expected = ControlPacket::Suback {
            packet_identifier: 4,
            reason_codes: vec![SubscribeReasonCode::GrantedQoS0],
        };
        assert_eq!(actual, expected);
        assert_eq!(remaining_bytes, 0);
    }

    #[test]
    fn test_encode_pingreq() {
        let packet = ControlPacket::Pingreq {};
        let actual = packet.encode().unwrap();
        let expected = [0xC0, 0x00];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decode_pingreq() {
        let packet = [0xC0, 0x00];
        let (actual, remaining_bytes) = ControlPacket::decode(&packet).unwrap();
        let expected = ControlPacket::Pingreq {};
        assert_eq!(actual, expected);
        assert_eq!(remaining_bytes, 0);
    }

    #[test]
    fn test_encode_pingresp() {
        let packet = ControlPacket::Pingresp {};
        let actual = packet.encode().unwrap();
        let expected = [0xD0, 0x00];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decode_pingresp() {
        let packet = [0xD0, 0x00];
        let (actual, remaining_bytes) = ControlPacket::decode(&packet).unwrap();
        let expected = ControlPacket::Pingresp {};
        assert_eq!(actual, expected);
        assert_eq!(remaining_bytes, 0);
    }

    #[test]
    fn test_encode_disconnect() {
        let packet = ControlPacket::Disconnect {
            reason_code: DisconnectReasonCode::NormalDisconnection,
        };
        let actual = packet.encode().unwrap();
        let expected = [0xE0, 0x00];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decode_disconnect() {
        let packet = [0xE0, 0x00];
        let (actual, remaining_bytes) = ControlPacket::decode(&packet).unwrap();
        let expected = ControlPacket::Disconnect {
            reason_code: DisconnectReasonCode::NormalDisconnection,
        };
        assert_eq!(actual, expected);
        assert_eq!(remaining_bytes, 0);
    }
}

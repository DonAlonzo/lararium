use crate::{protocol::*, ConnectReasonCode, DisconnectReasonCode, QoS};
use derive_more::From;
use lararium::prelude::*;
use std::fmt;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

pub struct Client {
    stream: TcpStream,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub topic: Topic,
    pub payload: Value,
}

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Protocol(crate::protocol::Error),
    #[from]
    Io(std::io::Error),
    #[from]
    Serialization(ciborium::ser::Error<std::io::Error>),
    ConnectionLost,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> Result<(), fmt::Error> {
        write!(f, "{self:?}")
    }
}

impl Client {
    pub fn connect(
        host: &str,
        port: u16,
    ) -> Result<Self, Error> {
        let mut stream = TcpStream::connect((host, port))?;
        stream.write_all(
            &ControlPacket::Connect { clean_start: true }
                .encode()
                .unwrap(),
        )?;
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer)?;
        let (packet, _) = ControlPacket::decode(&buffer[..bytes_read]).unwrap();
        let ControlPacket::Connack { reason_code } = packet else {
            panic!();
        };
        if reason_code != ConnectReasonCode::Success {
            panic!();
        }
        stream.set_nonblocking(true)?;
        Ok(Self { stream })
    }

    pub fn poll_message(&mut self) -> Result<Option<Message>, Error> {
        let mut buffer = [0; 1024];
        let bytes_read = match self.stream.read(&mut buffer) {
            Ok(bytes_read) => bytes_read,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let (packet, _) = ControlPacket::decode(&buffer[..bytes_read])?;
        match packet {
            ControlPacket::Publish { topic, payload } => {
                let Ok(payload) = ciborium::de::from_reader(&payload[..]) else {
                    panic!("Received faulty payload.");
                };
                Ok(Some(Message { topic, payload }))
            }
            ControlPacket::Puback { .. } => {
                tracing::debug!("Published successfully");
                Ok(None)
            }
            ControlPacket::Suback { .. } => {
                tracing::debug!("Subscribed successfully");
                Ok(None)
            }
            _ => {
                panic!("Unexpected packet: {packet:?}");
            }
        }
    }

    pub fn publish(
        &mut self,
        topic: impl Into<Topic>,
        value: Value,
        qos: QoS,
    ) -> Result<(), Error> {
        let mut payload = Vec::new();
        ciborium::ser::into_writer(&value, &mut payload)?;
        self.stream.write_all(
            &ControlPacket::Publish {
                topic: topic.into(),
                payload,
            }
            .encode()
            .unwrap(),
        )?;
        Ok(())
    }

    pub fn subscribe(
        &mut self,
        topic: impl Into<Topic>,
        qos: QoS,
    ) -> Result<(), Error> {
        self.stream.write_all(
            &ControlPacket::Subscribe {
                topic: topic.into(),
                packet_identifier: 0,
            }
            .encode()
            .unwrap(),
        )?;
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<(), Error> {
        self.stream.write_all(
            &ControlPacket::Disconnect {
                reason_code: DisconnectReasonCode::NormalDisconnection,
            }
            .encode()
            .unwrap(),
        )?;
        Ok(())
    }
}

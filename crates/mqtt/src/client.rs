use crate::{protocol::*, ConnectReasonCode, DisconnectReasonCode, QoS};
use derive_more::From;
use lararium::prelude::*;
use std::fmt;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Client {
    writer: Arc<Mutex<OwnedWriteHalf>>,
    rx: flume::Receiver<Message>,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub topic: Topic,
    pub payload: Value,
}

#[derive(Debug, From)]
pub enum Error {
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
    pub async fn connect(
        host: &str,
        port: u16,
    ) -> Result<Self, Error> {
        let stream = TcpStream::connect((host, port)).await.unwrap();
        let (mut reader, mut writer) = stream.into_split();
        writer
            .write_all(
                &ControlPacket::Connect { clean_start: true }
                    .encode()
                    .unwrap(),
            )
            .await
            .unwrap();
        let mut buffer = [0; 1024];
        let bytes_read = reader.read(&mut buffer).await.unwrap();
        let (packet, _) = ControlPacket::decode(&buffer[..bytes_read]).unwrap();
        match packet {
            ControlPacket::Connack { reason_code } => {
                if reason_code == ConnectReasonCode::Success {
                    let (tx, rx) = flume::unbounded();
                    tokio::spawn({
                        async move {
                            loop {
                                let Ok(bytes_read) = reader.read(&mut buffer).await else {
                                    break;
                                };
                                let Ok((packet, _)) = ControlPacket::decode(&buffer[..bytes_read])
                                else {
                                    break;
                                };
                                match packet {
                                    ControlPacket::Publish { topic, payload } => {
                                        let Ok(payload) = ciborium::de::from_reader(&payload[..])
                                        else {
                                            tracing::error!("Received faulty payload.");
                                            break;
                                        };
                                        if let Err(_) =
                                            tx.send_async(Message { topic, payload }).await
                                        {
                                            tracing::error!("Dropped connection.");
                                            break;
                                        }
                                    }
                                    ControlPacket::Puback { .. } => {
                                        tracing::debug!("Published successfully");
                                    }
                                    ControlPacket::Suback { .. } => {
                                        tracing::debug!("Subscribed successfully");
                                    }
                                    _ => tracing::error!("Unexpected packet: {packet:?}"),
                                }
                            }
                        }
                    });
                    Ok(Self {
                        writer: Arc::new(Mutex::new(writer)),
                        rx,
                    })
                } else {
                    panic!("Connection failed");
                }
            }
            _ => panic!("Unexpected packet"),
        }
    }

    pub async fn poll_message(&self) -> Result<Message, Error> {
        Ok(self
            .rx
            .recv_async()
            .await
            .map_err(|_| Error::ConnectionLost)?)
    }

    pub async fn publish(
        &self,
        topic: Topic,
        value: Value,
        qos: QoS,
    ) -> Result<(), Error> {
        let mut payload = Vec::new();
        ciborium::ser::into_writer(&value, &mut payload)?;
        self.writer
            .lock()
            .await
            .write_all(&ControlPacket::Publish { topic, payload }.encode().unwrap())
            .await?;
        Ok(())
    }

    pub async fn subscribe(
        &self,
        topic: Topic,
        qos: QoS,
    ) -> Result<(), Error> {
        self.writer
            .lock()
            .await
            .write_all(
                &ControlPacket::Subscribe {
                    topic,
                    packet_identifier: 0,
                }
                .encode()
                .unwrap(),
            )
            .await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), Error> {
        self.writer
            .lock()
            .await
            .write_all(
                &ControlPacket::Disconnect {
                    reason_code: DisconnectReasonCode::NormalDisconnection,
                }
                .encode()
                .unwrap(),
            )
            .await?;
        Ok(())
    }
}

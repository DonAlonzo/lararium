use crate::{protocol::*, ConnectReasonCode, DisconnectReasonCode, QoS, Result};
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
    topic_name: String,
    payload: Vec<u8>,
}

impl Client {
    pub async fn connect(host: &str) -> Result<Self> {
        let mut stream = TcpStream::connect("127.0.0.1:1883").await.unwrap();
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
                                let bytes_read = reader.read(&mut buffer).await.unwrap();
                                let (packet, _) =
                                    ControlPacket::decode(&buffer[..bytes_read]).unwrap();
                                match packet {
                                    ControlPacket::Publish {
                                        topic_name,
                                        payload,
                                    } => {
                                        tx.send_async(Message {
                                            topic_name,
                                            payload,
                                        })
                                        .await
                                        .expect("Send failed");
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

    pub async fn poll_message(&self) -> Result<Message> {
        Ok(self.rx.recv_async().await.expect("No message"))
    }

    pub async fn publish(
        &self,
        topic_name: &str,
        payload: &[u8],
        qos: QoS,
    ) -> Result<()> {
        self.writer
            .lock()
            .await
            .write_all(
                &ControlPacket::Publish {
                    topic_name: topic_name.into(),
                    payload: payload.into(),
                }
                .encode()
                .unwrap(),
            )
            .await?;
        Ok(())
    }

    pub async fn subscribe(
        &self,
        topic_name: &str,
        qos: QoS,
    ) -> Result<()> {
        self.writer
            .lock()
            .await
            .write_all(
                &ControlPacket::Subscribe {
                    topic_name: topic_name.into(),
                    packet_identifier: 0,
                }
                .encode()
                .unwrap(),
            )
            .await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
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

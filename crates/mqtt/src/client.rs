use crate::{protocol::*, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub async fn connect(host: &str) -> Result<Self> {
        let mut stream = TcpStream::connect("127.0.0.1:1883").await.unwrap();
        stream
            .write_all(
                &ControlPacket::Connect { clean_start: true }
                    .encode()
                    .unwrap(),
            )
            .await
            .unwrap();
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer).await.unwrap();
        // encode to hex
        for i in 0..bytes_read {
            print!("{:02x} ", buffer[i]);
        }
        println!();
        let (packet, _) = ControlPacket::decode(&buffer[..bytes_read]).unwrap();
        match packet {
            ControlPacket::Connack { reason_code } => {
                if reason_code == ConnectReasonCode::Success {
                    Ok(Self { stream })
                } else {
                    panic!("Connection failed");
                }
            }
            _ => panic!("Unexpected packet"),
        }
    }

    pub async fn publish(
        &mut self,
        topic_name: &str,
        payload: &[u8],
        qos: QoS,
    ) -> Result<()> {
        self.stream
            .write_all(
                &ControlPacket::Publish {
                    topic_name: topic_name.into(),
                    payload: payload.into(),
                }
                .encode()
                .unwrap(),
            )
            .await
            .unwrap();
        let mut buffer = [0; 1024];
        let bytes_read = self.stream.read(&mut buffer).await.unwrap();
        // encode to hex
        for i in 0..bytes_read {
            print!("{:02x} ", buffer[i]);
        }
        let (packet, _) = ControlPacket::decode(&buffer[..bytes_read]).unwrap();
        match packet {
            ControlPacket::Puback { .. } => Ok(()),
            _ => panic!("Unexpected packet"),
        }
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        self.stream
            .write_all(
                &ControlPacket::Disconnect {
                    reason_code: DisconnectReasonCode::NormalDisconnection,
                }
                .encode()
                .unwrap(),
            )
            .await
            .unwrap();
        Ok(())
    }
}

use crate::{protocol::*, Result};
use bytes::{Buf, BytesMut};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct Server {
    tcp_listener: TcpListener,
    context: Arc<Context>,
}

impl Server {
    pub async fn bind(listen_address: SocketAddr) -> Result<Self> {
        Ok(Self {
            tcp_listener: TcpListener::bind(listen_address).await?,
            context: Arc::new(Context {}),
        })
    }

    pub async fn listen(&self) -> Result<()> {
        loop {
            let (mut socket, address) = self.tcp_listener.accept().await?;
            let context = self.context.clone();
            tokio::spawn(async move {
                if let Err(error) = context.handle_connection(&mut socket).await {
                    tracing::error!("Error handling connection from {address}: {error}");
                }
                tracing::debug!("Connection from {address} closed");
            });
        }
    }
}

struct Context {}

enum Action {
    Respond(ControlPacket),
    Continue,
    Disconnect,
}

impl Context {
    async fn handle_connection(
        &self,
        stream: &mut TcpStream,
    ) -> Result<()> {
        let mut buffer = BytesMut::with_capacity(4096);
        loop {
            stream.readable().await?;
            let mut read_buffer = [0; 1024];
            let bytes_read = stream.read(&mut read_buffer).await?;
            if bytes_read == 0 {
                break Ok(());
            }
            buffer.extend_from_slice(&read_buffer[..bytes_read]);
            loop {
                // encode to hex
                for byte in buffer.iter() {
                    print!("{:02x} ", byte);
                }
                println!();
                match ControlPacket::decode(&buffer[..]) {
                    Ok((packet, remaining_bytes)) => {
                        match self.handle_packet(packet).await {
                            Ok(Action::Respond(packet)) => match packet.encode() {
                                Ok(packet) => {
                                    stream.write_all(&packet).await?;
                                }
                                Err(error) => {
                                    tracing::error!("Error encoding packet: {error}");
                                    continue;
                                }
                            },
                            Ok(Action::Disconnect) => {
                                return Ok(());
                            }
                            Ok(Action::Continue) => {}
                            Err(error) => {
                                tracing::error!("Error handling packet: {error}");
                                continue;
                            }
                        }
                        if remaining_bytes == 0 {
                            buffer.clear();
                            break;
                        }
                        buffer.advance(buffer.len() - remaining_bytes);
                    }
                    Err(crate::protocol::Error::Incomplete) => {
                        break;
                    }
                    Err(crate::protocol::Error::Invalid) => {
                        return Ok(());
                    }
                    Err(error) => {
                        tracing::error!("Error parsing message: {error}");
                        break;
                    }
                }
            }
        }
    }

    async fn handle_packet(
        &self,
        packet: ControlPacket,
    ) -> Result<Action> {
        match packet {
            ControlPacket::Connect { clean_start } => {
                tracing::debug!("Received CONNECT packet");
                Ok(Action::Respond(ControlPacket::Connack {
                    reason_code: ConnectReasonCode::Success,
                }))
            }
            ControlPacket::Publish {
                topic_name,
                payload,
            } => {
                tracing::debug!("Received PUBLISH packet on topic {}", topic_name,);
                Ok(Action::Respond(ControlPacket::Puback {}))
            }
            ControlPacket::Subscribe {
                packet_identifier,
                topic_name,
            } => {
                tracing::debug!(
                    "Received SUBSCRIBE packet {} on topic {}",
                    packet_identifier,
                    topic_name,
                );
                Ok(Action::Respond(ControlPacket::Suback {
                    packet_identifier,
                    reason_codes: vec![SubscribeReasonCode::GrantedQoS0],
                }))
            }
            ControlPacket::Pingreq => {
                tracing::debug!("Received PINGREQ packet");
                Ok(Action::Respond(ControlPacket::Pingresp))
            }
            ControlPacket::Disconnect { reason_code } => {
                tracing::debug!("Received DISCONNECT packet");
                Ok(Action::Disconnect)
            }
            _ => todo!(),
        }
    }
}

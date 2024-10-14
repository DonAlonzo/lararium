use crate::{protocol::*, Result};
use bytes::{Buf, BytesMut};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct Server {
    context: Arc<Context>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            context: Arc::new(Context {}),
        }
    }

    pub async fn listen(
        &self,
        listen_address: SocketAddr,
    ) -> Result<()> {
        let listener = TcpListener::bind(listen_address).await?;
        loop {
            let (mut socket, addr) = listener.accept().await?;
            let context = self.context.clone();
            tokio::spawn(async move {
                if let Err(e) = context.handle_connection(&mut socket).await {
                    tracing::error!("Error handling connection from {}: {}", addr, e);
                }
            });
        }
    }
}

struct Context {}

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
                match ControlPacket::decode(&buffer[..]) {
                    Ok((packet, remaining_bytes)) => {
                        let response = self.handle_packet(packet).await;
                        match response {
                            Ok(response) => match response.encode() {
                                Ok(encoded) => {
                                    stream.write_all(&encoded).await?;
                                }
                                Err(error) => {
                                    tracing::error!("Error encoding packet: {error}");
                                    continue;
                                }
                            },
                            Err(error) => {
                                tracing::error!("Error handling packet: {error}");
                                continue;
                            }
                        }
                        if remaining_bytes == 0 {
                            buffer.clear();
                            break;
                        }
                        buffer.advance(buffer.remaining() - remaining_bytes);
                    }
                    Err(crate::protocol::Error::Incomplete) => {
                        break;
                    }
                    Err(e) => {
                        tracing::error!("Error parsing message: {:?}", e);
                        break;
                    }
                }
            }
        }
    }

    async fn handle_packet(
        &self,
        packet: ControlPacket,
    ) -> Result<ControlPacket> {
        match packet {
            ControlPacket::Connect {} => {
                tracing::info!("Received CONNECT packet");
                Ok(ControlPacket::Connack {
                    reason_code: ConnectReasonCode::Success,
                })
            }
            ControlPacket::Publish {
                topic_name,
                payload,
            } => {
                tracing::info!("Received PUBLISH packet: {} {:?}", topic_name, payload);
                Ok(ControlPacket::Puback {})
            }
            _ => todo!(),
        }
    }
}

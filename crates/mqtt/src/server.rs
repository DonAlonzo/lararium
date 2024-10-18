use crate::{
    protocol::*, Connack, Connect, ConnectReasonCode, Disconnect, Puback, Publish, Result, Suback,
    Subscribe, SubscribeReasonCode,
};
use bytes::{Buf, BytesMut};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct Server {
    tcp_listener: TcpListener,
}

pub trait Handler {
    fn handle_connect(
        &self,
        connect: Connect,
    ) -> impl std::future::Future<Output = Connack> + Send;

    fn handle_disconnect(
        &self,
        disconnect: Disconnect,
    ) -> impl std::future::Future<Output = ()> + Send;

    fn handle_ping(&self) -> impl std::future::Future<Output = ()> + Send;

    fn handle_publish(
        &self,
        publish: Publish,
    ) -> impl std::future::Future<Output = Puback> + Send;

    fn handle_subscribe(
        &self,
        subscribe: Subscribe,
    ) -> impl std::future::Future<Output = Suback> + Send;
}

impl Server {
    pub async fn bind(listen_address: SocketAddr) -> Result<Self> {
        Ok(Self {
            tcp_listener: TcpListener::bind(listen_address).await?,
        })
    }

    pub async fn listen<T>(
        &self,
        handler: T,
    ) -> Result<()>
    where
        T: Handler + Clone + Send + Sync + 'static,
    {
        loop {
            let (mut socket, address) = self.tcp_listener.accept().await?;
            let handler = handler.clone();
            tokio::spawn(async move {
                if let Err(error) = handle_connection(&mut socket, handler).await {
                    tracing::error!("Error handling connection from {address}: {error}");
                }
                tracing::debug!("Connection from {address} closed");
            });
        }
    }
}

enum Action {
    Respond(ControlPacket),
    Continue,
    Disconnect,
}

async fn handle_connection<T>(
    stream: &mut TcpStream,
    handler: T,
) -> Result<()>
where
    T: Handler,
{
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
            match ControlPacket::decode(&buffer[..]) {
                Ok((packet, remaining_bytes)) => {
                    match handle_packet(packet, &handler).await {
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

async fn handle_packet<T>(
    packet: ControlPacket,
    handler: &T,
) -> Result<Action>
where
    T: Handler,
{
    match packet {
        ControlPacket::Connect { clean_start } => {
            let connack = handler.handle_connect(Connect { clean_start }).await;
            Ok(Action::Respond(ControlPacket::Connack {
                reason_code: connack.reason_code,
            }))
        }
        ControlPacket::Publish {
            topic_name,
            payload,
        } => {
            let _puback = handler
                .handle_publish(Publish {
                    topic_name: &topic_name,
                    payload: &payload,
                })
                .await;
            Ok(Action::Respond(ControlPacket::Puback {}))
        }
        ControlPacket::Subscribe {
            packet_identifier,
            topic_name,
        } => {
            let suback = handler
                .handle_subscribe(Subscribe {
                    topic_name: &topic_name,
                })
                .await;
            Ok(Action::Respond(ControlPacket::Suback {
                packet_identifier,
                reason_codes: suback.reason_codes.to_vec(),
            }))
        }
        ControlPacket::Pingreq => {
            handler.handle_ping().await;
            Ok(Action::Respond(ControlPacket::Pingresp))
        }
        ControlPacket::Disconnect { reason_code } => {
            handler.handle_disconnect(Disconnect { reason_code }).await;
            Ok(Action::Disconnect)
        }
        _ => todo!(),
    }
}

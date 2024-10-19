use crate::{protocol::*, *};
use bytes::{Buf, BytesMut};
use flume::Sender;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpListener, TcpStream,
};
use tokio::sync::Mutex;

pub struct Server {
    tcp_listener: TcpListener,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Connect {
    pub clean_start: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Connack {
    pub reason_code: ConnectReasonCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Publish<'a> {
    pub topic_name: &'a str,
    pub payload: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Puback {}

#[derive(Debug, Clone)]
pub struct Subscribe<'a> {
    pub topic_name: &'a str,
    pub tx: Sender<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Suback<'a> {
    pub reason_codes: &'a [SubscribeReasonCode],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Disconnect {
    pub reason_code: DisconnectReasonCode,
}

enum Action {
    Respond(ControlPacket),
    Continue,
    Disconnect,
}

pub trait Handler {
    fn handle_connect(
        &mut self,
        connect: Connect,
    ) -> impl std::future::Future<Output = Connack> + Send;

    fn handle_disconnect(
        &mut self,
        disconnect: Disconnect,
    ) -> impl std::future::Future<Output = ()> + Send;

    fn handle_ping(&mut self) -> impl std::future::Future<Output = ()> + Send;

    fn handle_publish(
        &mut self,
        publish: Publish,
    ) -> impl std::future::Future<Output = Puback> + Send;

    fn handle_subscribe(
        &mut self,
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
            let (stream, address) = self.tcp_listener.accept().await?;
            let handler = handler.clone();
            tokio::spawn(async move {
                let (reader, writer) = stream.into_split();
                let writer = Arc::new(Mutex::new(writer));
                if let Err(error) = handle_connection(reader, writer, handler).await {
                    tracing::error!("Error handling connection from {address}: {error}");
                }
                tracing::debug!("Connection from {address} closed");
            });
        }
    }
}

async fn handle_connection<T>(
    mut reader: OwnedReadHalf,
    writer: Arc<Mutex<OwnedWriteHalf>>,
    mut handler: T,
) -> Result<()>
where
    T: Handler,
{
    let mut buffer = BytesMut::with_capacity(4096);
    loop {
        let mut read_buffer = [0; 1024];
        let bytes_read = {
            reader.readable().await?;
            let bytes_read = reader.read(&mut read_buffer).await?;
            if bytes_read == 0 {
                break Ok(());
            }
            bytes_read
        };
        buffer.extend_from_slice(&read_buffer[..bytes_read]);
        loop {
            match ControlPacket::decode(&buffer[..]) {
                Ok((packet, remaining_bytes)) => {
                    match handle_packet(&writer, packet, &mut handler).await {
                        Ok(Action::Respond(packet)) => match packet.encode() {
                            Ok(packet) => {
                                let mut writer = writer.lock().await;
                                writer.write_all(&packet).await?;
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
    writer: &Arc<Mutex<OwnedWriteHalf>>,
    packet: ControlPacket,
    handler: &mut T,
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
            let (tx, rx) = flume::bounded(32);
            let suback = handler
                .handle_subscribe(Subscribe {
                    topic_name: &topic_name,
                    tx,
                })
                .await;
            tokio::spawn({
                let writer = writer.clone();
                async move {
                    while let Ok(payload) = rx.recv_async().await {
                        let control_packet = ControlPacket::Publish {
                            topic_name: topic_name.clone(),
                            payload,
                        };
                        let packet = control_packet.encode().unwrap();
                        let mut writer = writer.lock().await;
                        writer.write_all(&packet).await.unwrap();
                    }
                }
            });
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

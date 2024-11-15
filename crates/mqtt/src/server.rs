use crate::{protocol::*, *};
use bytes::{Buf, BytesMut};
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpListener, TcpStream,
};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Server<T>
where
    T: Handler,
{
    tcp_listener: Arc<TcpListener>,
    next_client_id: Arc<AtomicU64>,
    connections: Arc<DashMap<u64, Connection<T>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Connect {
    pub client_id: u64,
    pub clean_start: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Disconnect {
    pub client_id: u64,
    pub reason_code: DisconnectReasonCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Publish<'a> {
    pub topic_name: &'a str,
    pub payload: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subscribe<'a> {
    pub client_id: u64,
    pub topic_name: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Connack {
    pub reason_code: ConnectReasonCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Puback {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suback {
    pub reason_codes: Vec<SubscribeReasonCode>,
}

enum Action {
    Respond(ControlPacket),
    Continue,
    Disconnect,
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

impl<T> Server<T>
where
    T: Handler + Clone + Send + Sync + 'static,
{
    pub async fn bind(listen_address: SocketAddr) -> Result<Self> {
        Ok(Self {
            tcp_listener: Arc::new(TcpListener::bind(listen_address).await?),
            next_client_id: Arc::new(AtomicU64::new(0)),
            connections: Arc::new(DashMap::new()),
        })
    }

    pub async fn listen(
        &self,
        handler: T,
    ) -> Result<()> {
        loop {
            let (stream, address) = self.tcp_listener.accept().await?;
            let handler = handler.clone();
            let client_id = self.next_client_id.fetch_add(1, Ordering::SeqCst);
            let (reader, writer) = stream.into_split();
            let reader = Arc::new(Mutex::new(reader));
            let writer = Arc::new(Mutex::new(writer));
            let connection = Connection {
                client_id,
                reader: reader.clone(),
                writer: writer.clone(),
                handler,
            };
            self.connections.insert(client_id, connection.clone());
            let this = self.clone();
            tokio::spawn(async move {
                if let Err(error) = connection.handle().await {
                    tracing::error!("Error handling connection from {address}: {error}");
                }
                tracing::debug!("Connection from {address} closed");
                this.connections.remove(&client_id);
            });
        }
    }

    pub async fn publish(
        &self,
        client_ids: &[u64],
        topic_name: &str,
        payload: &[u8],
    ) -> Result<()> {
        for client_id in client_ids {
            if let Some(connection) = self.connections.get(client_id) {
                connection.publish(topic_name, payload).await?;
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
struct Connection<T>
where
    T: Handler,
{
    client_id: u64,
    reader: Arc<Mutex<OwnedReadHalf>>,
    writer: Arc<Mutex<OwnedWriteHalf>>,
    handler: T,
}

impl<T> Connection<T>
where
    T: Handler,
{
    async fn publish(
        &self,
        topic_name: &str,
        payload: &[u8],
    ) -> Result<()> {
        tracing::debug!("Publishing to {}: {topic_name}", self.client_id);
        Ok(())
    }

    async fn handle(&self) -> Result<()> {
        let mut buffer = BytesMut::with_capacity(4096);
        loop {
            let mut read_buffer = [0; 1024];
            let bytes_read = {
                let mut reader = self.reader.lock().await;
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
                        match self.handle_packet(packet).await {
                            Ok(Action::Respond(packet)) => match packet.encode() {
                                Ok(packet) => {
                                    let mut writer = self.writer.lock().await;
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

    async fn handle_packet(
        &self,
        packet: ControlPacket,
    ) -> Result<Action>
    where
        T: Handler,
    {
        match packet {
            ControlPacket::Connect { clean_start } => {
                let connack = self
                    .handler
                    .handle_connect(Connect {
                        client_id: self.client_id,
                        clean_start,
                    })
                    .await;
                Ok(Action::Respond(ControlPacket::Connack {
                    reason_code: connack.reason_code,
                }))
            }
            ControlPacket::Publish {
                topic_name,
                payload,
            } => {
                let _puback = self
                    .handler
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
                let suback = self
                    .handler
                    .handle_subscribe(Subscribe {
                        client_id: self.client_id,
                        topic_name: &topic_name,
                    })
                    .await;
                Ok(Action::Respond(ControlPacket::Suback {
                    packet_identifier,
                    reason_codes: suback.reason_codes.to_vec(),
                }))
            }
            ControlPacket::Pingreq => {
                self.handler.handle_ping().await;
                Ok(Action::Respond(ControlPacket::Pingresp))
            }
            ControlPacket::Disconnect { reason_code } => {
                self.handler
                    .handle_disconnect(Disconnect {
                        client_id: self.client_id,
                        reason_code,
                    })
                    .await;
                Ok(Action::Disconnect)
            }
            _ => todo!(),
        }
    }
}

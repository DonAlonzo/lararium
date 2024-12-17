use crate::{protocol::*, *};
use bytes::{Buf, BytesMut};
use dashmap::DashMap;
use derive_more::From;
use lararium::prelude::*;
use std::fmt;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpListener,
};
use tokio::sync::Mutex;

type ClientId = u64;

#[derive(Clone)]
pub struct Server<T>
where
    T: Handler,
{
    tcp_listener: Arc<TcpListener>,
    next_client_id: Arc<AtomicU64>,
    connections: Arc<DashMap<ClientId, Connection<T>>>,
}

#[derive(Clone)]
struct Connection<T>
where
    T: Handler,
{
    client_id: ClientId,
    reader: Arc<Mutex<OwnedReadHalf>>,
    writer: Arc<Mutex<OwnedWriteHalf>>,
    handler: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Connect {
    pub client_id: ClientId,
    pub clean_start: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Disconnect {
    pub client_id: ClientId,
    pub reason_code: DisconnectReasonCode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Publish {
    pub client_id: ClientId,
    pub topic: Topic,
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscribe {
    pub client_id: ClientId,
    pub filter: Filter,
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

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Io(std::io::Error),
    #[from]
    Deserialization(ciborium::de::Error<std::io::Error>),
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
    pub async fn bind(listen_address: SocketAddr) -> Result<Self, Error> {
        Ok(Self {
            tcp_listener: Arc::new(TcpListener::bind(listen_address).await?),
            next_client_id: Arc::new(AtomicU64::new(0)),
            connections: Arc::new(DashMap::new()),
        })
    }

    pub async fn listen(
        &self,
        handler: T,
    ) -> Result<(), Error> {
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
            tokio::spawn({
                let connection = connection.clone();
                async move {
                    if let Err(error) = connection.read().await {
                        tracing::error!("Error handling connection from {address}: {error}");
                    }
                    tracing::debug!("Connection from {address} closed");
                    this.connections.remove(&client_id);
                }
            });
        }
    }

    pub async fn publish(
        &self,
        client_ids: &[ClientId],
        topic: &Topic,
        payload: Option<Value>,
    ) -> Result<(), Error> {
        for client_id in client_ids {
            if let Some(connection) = self.connections.get(client_id) {
                connection.publish(topic.clone(), payload.clone()).await?;
            }
        }
        Ok(())
    }
}

impl<T> Connection<T>
where
    T: Handler,
{
    async fn publish(
        &self,
        topic: Topic,
        value: Option<Value>,
    ) -> Result<(), Error> {
        tracing::debug!("Publishing to {}: {topic}", self.client_id);
        let mut payload = Vec::new();
        if let Some(value) = value {
            ciborium::ser::into_writer(&value, &mut payload).unwrap();
        }
        let packet = ControlPacket::Publish { topic, payload };
        self.write(packet).await
    }

    async fn write(
        &self,
        packet: ControlPacket,
    ) -> Result<(), Error> {
        let packet = packet.encode().unwrap();
        let mut writer = self.writer.lock().await;
        writer.write_all(&packet).await?;
        Ok(())
    }

    async fn read(&self) -> Result<(), Error> {
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
                            Ok(Action::Respond(packet)) => {
                                self.write(packet).await.unwrap();
                            }
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
    ) -> Result<Action, Error>
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
            ControlPacket::Publish { topic, payload } => {
                let payload = if payload.len() == 0 {
                    None
                } else {
                    Some(ciborium::de::from_reader::<Value, _>(&payload[..])?)
                };
                let _puback = self
                    .handler
                    .handle_publish(Publish {
                        client_id: self.client_id,
                        topic,
                        payload,
                    })
                    .await;
                Ok(Action::Respond(ControlPacket::Puback {}))
            }
            ControlPacket::Subscribe {
                packet_identifier,
                topic,
            } => {
                let suback = self
                    .handler
                    .handle_subscribe(Subscribe {
                        client_id: self.client_id,
                        filter: topic.into(),
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

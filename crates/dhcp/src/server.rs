use crate::{protocol::*, HardwareType, Result};
use bytes::BytesMut;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

#[derive(Clone)]
pub struct Server {
    udp_socket: Arc<UdpSocket>,
}

#[derive(Clone, Debug)]
pub struct Discover {}

#[derive(Clone, Debug)]
pub struct Offer {}

pub trait Handler {
    fn handle_discover(
        &self,
        discover: Discover,
    ) -> impl std::future::Future<Output = Option<Offer>> + Send;
}

impl Server {
    pub async fn bind(listen_address: SocketAddr) -> Result<Self> {
        Ok(Self {
            udp_socket: Arc::new(UdpSocket::bind(listen_address).await?),
        })
    }

    pub async fn listen<T>(
        &self,
        handler: T,
    ) -> Result<()>
    where
        T: Handler + Clone + Send + Sync + 'static,
    {
        let mut buffer = BytesMut::with_capacity(1024);
        buffer.resize(1024, 0);
        loop {
            match self.udp_socket.recv_from(&mut buffer).await {
                Ok((bytes_read, address)) => {
                    let Some(message) = buffer.get_message() else {
                        continue;
                    };
                    let handler = handler.clone();
                    tokio::spawn(async move {
                        let Some(offer) = handler.handle_discover(Discover {}).await else {
                            return;
                        };
                    });
                }
                Err(error) => {
                    eprintln!("Error receiving data: {error}");
                }
            }
        }
    }
}

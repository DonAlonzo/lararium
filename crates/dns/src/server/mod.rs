mod error;
mod handler;

pub use error::Error;
pub use handler::Handler;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, UdpSocket};

#[derive(Clone)]
pub struct Server {
    udp_socket: Arc<UdpSocket>,
    tcp_listener: Arc<TcpListener>,
}

impl Server {
    pub async fn bind(listen_address: SocketAddr) -> Result<Self, Error> {
        Ok(Self {
            udp_socket: Arc::new(UdpSocket::bind(listen_address).await?),
            tcp_listener: Arc::new(TcpListener::bind(listen_address).await?),
        })
    }

    pub async fn listen<T>(
        &self,
        handler: T,
    ) -> Result<(), Error>
    where
        T: Handler + Clone + Send + Sync + 'static,
    {
        tokio::select!(
            result = self.listen_udp(handler.clone()) => result?,
            result = self.listen_tcp(handler) => result?,
        );
        Ok(())
    }

    async fn listen_udp<T>(
        &self,
        handler: T,
    ) -> Result<(), Error>
    where
        T: Handler,
    {
        loop {
            let mut buffer = [0; 512];
            let (size, address) = self.udp_socket.recv_from(&mut buffer).await?;
            let message = &buffer[..size];
            tracing::debug!("DNS/UDP: {message:?}");
            // self.udp_socket.send_to(&response, address).await?;
        }
    }

    async fn listen_tcp<T>(
        &self,
        handler: T,
    ) -> Result<(), Error>
    where
        T: Handler + Clone + Send + Sync + 'static,
    {
        loop {
            let (mut socket, address) = self.tcp_listener.accept().await?;
            let handler = handler.clone();
            tracing::debug!("DNS/TCP: {address}");
            tokio::spawn(async move {});
        }
    }
}

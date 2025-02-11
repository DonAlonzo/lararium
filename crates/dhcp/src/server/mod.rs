mod error;
mod handler;

pub use error::Error;
pub use handler::Handler;

use bytes::BytesMut;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

#[derive(Clone)]
pub struct Server {
    udp_socket: Arc<UdpSocket>,
}

impl Server {
    pub async fn bind(listen_address: SocketAddr) -> Result<Self, Error> {
        Ok(Self {
            udp_socket: Arc::new(UdpSocket::bind(listen_address).await?),
        })
    }

    pub async fn listen<T>(
        &self,
        handler: T,
    ) -> Result<(), Error>
    where
        T: Handler + Clone + Send + Sync + 'static,
    {
        let mut buffer = BytesMut::with_capacity(1024);
        buffer.resize(1024, 0);
        loop {
            let Ok((bytes_read, address)) = self.udp_socket.recv_from(&mut buffer).await else {
                tracing::error!("Error receiving data.");
                continue;
            };
            let message = &buffer[..bytes_read];
            tracing::debug!("DHCP: {message:?}");
        }
    }
}

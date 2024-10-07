mod error;

pub use self::error::{Error, Result};

//use ed25519_dalek::{Signature, Signer, SigningKey};
//use rand::rngs::OsRng;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use tokio::net::UdpSocket;

const ANNOUNCEMENT_MESSAGE: &[u8] = b"LARARIUM_ANNOUNCEMENT";

enum Protocol {
    IPv4,
    IPv6,
}

pub struct Client {}

pub struct Server {}

impl Client {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn announce(
        &self,
        port: u16,
    ) -> Result<()> {
        let socket_v4 = UdpSocket::bind::<SocketAddr>((Ipv4Addr::UNSPECIFIED, port).into()).await?;
        let socket_v6 = UdpSocket::bind::<SocketAddr>((Ipv6Addr::UNSPECIFIED, port).into()).await?;
        socket_v4.set_broadcast(true)?;
        socket_v6.set_broadcast(true)?;

        loop {
            match socket_v4
                .send_to::<SocketAddr>(ANNOUNCEMENT_MESSAGE, (Ipv4Addr::BROADCAST, port).into())
                .await
            {
                Ok(_) => tracing::debug!("Sent IPv4 announcement"),
                Err(error) => tracing::error!("Error sending IPv4 announcement: {}", error),
            }
            match socket_v6
                .send_to::<SocketAddr>(
                    ANNOUNCEMENT_MESSAGE,
                    (Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 1), port).into(),
                )
                .await
            {
                Ok(_) => tracing::debug!("Sent IPv4 announcement"),
                Err(error) => tracing::error!("Error sending IPv6 announcement: {}", error),
            }
        }
    }
}

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    async fn handle_message(
        &self,
        buffer: &[u8],
        source: std::net::SocketAddr,
        protocol: Protocol,
    ) {
        println!(
            "Received {} message from {}: {}",
            match protocol {
                Protocol::IPv4 => "IPv4",
                Protocol::IPv6 => "IPv6",
            },
            source,
            buffer.len()
        );
    }

    pub async fn listen_for_announcements(
        &self,
        port: u16,
    ) -> Result<()> {
        let socket_v4 = UdpSocket::bind::<SocketAddr>((Ipv4Addr::UNSPECIFIED, port).into()).await?;
        let socket_v6 = UdpSocket::bind::<SocketAddr>((Ipv6Addr::UNSPECIFIED, port).into()).await?;
        socket_v4.set_broadcast(true)?;
        socket_v6.set_broadcast(true)?;
        let mut buffer_v4 = [0u8; 1024];
        let mut buffer_v6 = [0u8; 1024];
        loop {
            tokio::select! {
                Ok((size, source)) = socket_v4.recv_from(&mut buffer_v4) => {
                    self.handle_message(&buffer_v4[..size], source, Protocol::IPv4).await;
                }
                Ok((size, source)) = socket_v6.recv_from(&mut buffer_v6) => {
                    self.handle_message(&buffer_v6[..size], source, Protocol::IPv6).await;
                }
            }
        }
    }
}

//pub fn sign(data: &[u8]) -> Signature {
//    let mut prng = OsRng;
//    let signing_key: SigningKey = SigningKey::generate(&mut prng);
//    signing_key.sign(data)
//}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_() {}
}

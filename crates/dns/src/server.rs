use crate::{Query, Response, Result};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, UdpSocket};

pub struct Server {
    udp_socket: UdpSocket,
    tcp_listener: TcpListener,
}

pub trait Handler {
    fn handle_dns_query(
        &self,
        query: &Query,
    ) -> impl std::future::Future<Output = Option<Response>> + Send;
}

impl Server {
    pub async fn bind(listen_address: SocketAddr) -> Result<Self> {
        Ok(Self {
            udp_socket: UdpSocket::bind(listen_address).await?,
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
        tokio::select!(
            result = self.listen_udp(handler.clone()) => result?,
            result = self.listen_tcp(handler) => result?,
        );
        Ok(())
    }

    async fn listen_udp<T>(
        &self,
        handler: T,
    ) -> std::io::Result<()>
    where
        T: Handler,
    {
        loop {
            let mut buffer = [0; 512];
            let (size, address) = self.udp_socket.recv_from(&mut buffer).await?;
            let query = Query::decode(&buffer[..size]);
            if let Some(response) = handler.handle_dns_query(&query).await {
                let response = response.encode(&query);
                self.udp_socket.send_to(&response, address).await?;
            }
        }
    }

    async fn listen_tcp<T>(
        &self,
        handler: T,
    ) -> std::io::Result<()>
    where
        T: Handler + Clone + Send + Sync + 'static,
    {
        loop {
            let (mut socket, _address) = self.tcp_listener.accept().await?;
            let handler = handler.clone();
            tokio::spawn(async move {
                let mut length_buffer = [0; 2];
                if socket.read_exact(&mut length_buffer).await.is_err() {
                    return;
                }
                let query_length = u16::from_be_bytes(length_buffer) as usize;

                let mut query_buffer = vec![0; query_length];
                if socket.read_exact(&mut query_buffer).await.is_err() {
                    return;
                }
                let query = Query::decode(&query_buffer);
                if let Some(response) = handler.handle_dns_query(&query).await {
                    let response = response.encode(&query);
                    let response_length = (response.len() as u16).to_be_bytes();
                    if let Err(error) = socket.write_all(&response_length).await {
                        eprintln!("Failed to write response length: {error}");
                        return;
                    };
                    if let Err(error) = socket.write_all(&response).await {
                        eprintln!("Failed to write response: {error}");
                        return;
                    };
                }
            });
        }
    }
}

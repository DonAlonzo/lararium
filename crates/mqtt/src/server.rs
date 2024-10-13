use crate::{protocol::Packet, Result};
use bytes::{Buf, BytesMut};
use deku::prelude::*;
use std::net::SocketAddr;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn listen(
        &self,
        listen_address: SocketAddr,
    ) -> Result<()> {
        let listener = TcpListener::bind(listen_address).await?;
        loop {
            let (mut socket, addr) = listener.accept().await?;
            tokio::spawn(async move {
                if let Err(e) = handle_connection(&mut socket).await {
                    tracing::error!("Error handling connection from {}: {}", addr, e);
                }
            });
        }
    }
}

async fn handle_connection(stream: &mut TcpStream) -> Result<()> {
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
            match Packet::from_bytes((&buffer[..], 0)) {
                Ok((rest, packet)) => {
                    tracing::info!("{:?}", packet);
                    buffer.advance(buffer.len() - rest.0.len());
                }
                Err(deku::error::DekuError::Incomplete(_)) => {
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

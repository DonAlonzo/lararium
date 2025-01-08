mod protocol;

use crate::protocol::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use derive_more::From;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct Server {
    listener: Arc<TcpListener>,
}

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Io(std::io::Error),
}

impl std::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter,
    ) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl Server {
    pub async fn bind(listen_address: SocketAddr) -> Result<Self, Error> {
        Ok(Self {
            listener: Arc::new(TcpListener::bind(listen_address).await?),
        })
    }

    pub async fn listen(&self) -> Result<(), Error> {
        loop {
            let (mut socket, address) = self.listener.accept().await?;
            tracing::debug!("Received connection from {address}.");
            tokio::spawn(async move {
                let mut buffer = [0; 1024];
                loop {
                    if socket.read_exact(&mut buffer[..4]).await.is_err() {
                        break;
                    }
                    let mut record_mark = &buffer[..4];
                    let record_mark = record_mark.get_u32();
                    let last_fragment = record_mark & (1 << 31) != 0;
                    let length = (record_mark & ((1 << 31) - 1)) as usize;
                    if socket.read_exact(&mut buffer[..length]).await.is_err() {
                        break;
                    }
                    let mut message = &buffer[..length];
                    let transaction_id = message.get_u32();
                    let message_type = message.get_u32();

                    match message_type {
                        0 => {
                            let rpc_version = message.get_u32();
                            if rpc_version != 2 {
                                break;
                            }

                            let program_number = message.get_u32();
                            if program_number != 100003 {
                                break;
                            }

                            let program_version = message.get_u32();
                            if program_version != 4 {
                                break;
                            }

                            let procedure_number = message.get_u32();

                            let opaque_auth_cred_flavor = message.get_u32();
                            let opaque_auth_cred_body_length = message.get_u32() as usize;
                            message.advance(opaque_auth_cred_body_length);

                            let opaque_auth_verf_flavor = message.get_u32();
                            let opaque_auth_verf_body_length = message.get_u32() as usize;
                            message.advance(opaque_auth_verf_body_length);

                            let reply_body = match procedure_number {
                                // NULL
                                0 => Bytes::new(),
                                // COMPOUND
                                1 => {
                                    let args = compound4_args(&message).unwrap();
                                    Bytes::new()
                                }
                                _ => break,
                            };
                            let mut reply = BytesMut::with_capacity(32 + reply_body.len());
                            reply.put_u32(1 << 31 | (28 + reply_body.len() as u32));
                            reply.put_u32(transaction_id);
                            reply.put_u32(1); // msg_type = REPLY
                            reply.put_u32(0); // status = accepted
                            reply.put_u32(0); // verifier flavor
                            reply.put_u32(0); // verifier body length
                            reply.put_u32(0); // accept status = success
                            reply.put_u32(reply_body.len() as u32);
                            reply.put(reply_body);
                            if socket.write_all(&reply).await.is_err() {
                                break;
                            }
                        }
                        1 => {
                            todo!();
                        }
                        _ => {
                            tracing::error!("8");
                            break;
                        }
                    };
                }
                tracing::debug!("Connection to {address} lost.");
                Ok::<_, Error>(())
            });
        }
        Ok(())
    }
}

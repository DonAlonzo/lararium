mod connection;
mod handler;

pub use handler::Handler;

use crate::protocol::{self, *};
use connection::Connection;
use cookie_factory::{gen, sequence::tuple};
use derive_more::From;
use std::io::{self, Cursor};
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

    pub async fn listen<T>(
        &self,
        handler: T,
    ) -> Result<(), Error>
    where
        T: Handler + Clone + Send + Sync + 'static,
    {
        loop {
            let (mut socket, address) = self.listener.accept().await?;
            tracing::debug!("Received connection from {address}.");
            let connection = Connection::new(handler.clone());
            tokio::spawn({
                async move {
                    let mut buffer = [0; 1024];
                    loop {
                        let record_mark = match socket.read_u32().await {
                            Ok(record_mark) => record_mark,
                            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => break,
                            Err(error) => {
                                tracing::debug!("Failed to read record mark: {error}");
                                break;
                            }
                        };
                        let last_fragment = record_mark & (1 << 31) != 0;
                        let fragment_length = (record_mark & ((1 << 31) - 1)) as usize;
                        if socket
                            .read_exact(&mut buffer[..fragment_length])
                            .await
                            .is_err()
                        {
                            tracing::debug!("Failed to read record fragment.");
                            break;
                        }
                        let input = &buffer[..fragment_length];
                        let Ok((input, RpcMessage { xid, message_type })) =
                            protocol::decode::message(input)
                        else {
                            tracing::debug!("Invalid RPC message.");
                            continue;
                        };
                        match message_type {
                            MessageType::Call => {
                                let (input, call) = match protocol::decode::call(input) {
                                    Ok(value) => value,
                                    Err(error) => {
                                        tracing::debug!("Invalid RPC call: {error}");
                                        continue;
                                    }
                                };
                                let reply = match call.procedure {
                                    ProcedureCall::Null => {
                                        tracing::debug!("Received null call.");
                                        ProcedureReply::Null
                                    }
                                    ProcedureCall::Compound(args) => {
                                        tracing::debug!("Received compound call.");
                                        // TODO args.minorversion
                                        let mut resarray = Vec::with_capacity(args.argarray.len());
                                        for nfs_argop in args.argarray.into_iter() {
                                            resarray.push(match nfs_argop {
                                                NfsArgOp::PutRootFileHandle => {
                                                    NfsResOp::PutRootFileHandle(
                                                        connection.put_root_file_handle().await,
                                                    )
                                                }
                                                // NfsArgOp::SecInfo(args) => todo!(),
                                                NfsArgOp::ExchangeId(args) => NfsResOp::ExchangeId(
                                                    connection.exchange_id(args).await,
                                                ),
                                                NfsArgOp::CreateSession(args) => {
                                                    NfsResOp::CreateSession(
                                                        connection.create_session(args).await,
                                                    )
                                                }
                                                NfsArgOp::DestroySession(args) => {
                                                    NfsResOp::DestroySession(
                                                        connection.destroy_session(args).await,
                                                    )
                                                }
                                                NfsArgOp::DestroyClientId(args) => {
                                                    NfsResOp::DestroyClientId(
                                                        connection.destroy_client_id(args).await,
                                                    )
                                                }
                                                // NfsArgOp::SecInfoNoName(args) => todo!(),
                                                NfsArgOp::Sequence(args) => NfsResOp::Sequence(
                                                    connection.sequence(args).await,
                                                ),
                                                NfsArgOp::ReclaimComplete(args) => {
                                                    NfsResOp::ReclaimComplete(
                                                        connection.reclaim_complete(args).await,
                                                    )
                                                }
                                            });
                                        }
                                        ProcedureReply::Compound(CompoundResult {
                                            error: None,
                                            tag: args.tag,
                                            resarray,
                                        })
                                    }
                                };
                                let mut buffer = [0; 1024];
                                let output = {
                                    let generator = tuple((
                                        protocol::encode::message(RpcMessage {
                                            xid,
                                            message_type: MessageType::Reply,
                                        }),
                                        protocol::encode::reply(Reply::Accepted(AcceptedReply {
                                            verf: OpaqueAuth {
                                                flavor: AuthFlavor::AuthNone, // TODO
                                                body: (&[]).into(),           // TODO
                                            },
                                            body: AcceptedReplyBody::Success(reply),
                                        })),
                                    ));
                                    let cursor = Cursor::new(&mut buffer[..]);
                                    let Ok((_, position)) = gen(generator, cursor) else {
                                        tracing::debug!("Failed to encode reply.");
                                        continue;
                                    };
                                    &buffer[..position as usize]
                                };
                                let last_fragment = true;
                                let record_mark = if last_fragment {
                                    output.len() as u32 | 1 << 31
                                } else {
                                    output.len() as u32
                                };
                                if socket.write_u32(record_mark).await.is_err() {
                                    break;
                                }
                                if socket.write_all(output).await.is_err() {
                                    break;
                                }
                            }
                            MessageType::Reply => break,
                        }
                    }
                    tracing::debug!("Connection to {address} lost.");
                    Ok::<_, Error>(())
                }
            });
        }
        Ok(())
    }
}

use crate::protocol::{self, *};
use cookie_factory::gen;
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

#[derive(Clone)]
pub struct Connection<T>
where
    T: Handler + Clone + Send + Sync + 'static,
{
    handler: T,
}

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Io(std::io::Error),
}

pub trait Handler {
    // fn call(
    //     &self,
    //     args: Args,
    // ) -> impl std::future::Future<Output = Result> + Send;
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
            let connection = Connection {
                handler: handler.clone(),
            };
            tokio::spawn({
                async move {
                    let mut buffer = [0; 1024];
                    loop {
                        let record_mark = match socket.read_u32().await {
                            Ok(record_mark) => record_mark,
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
                        let message = &buffer[..fragment_length];
                        let Ok((_, RpcMessage { xid, message })) = protocol::decode(message) else {
                            tracing::debug!("Invalid RPC message.");
                            break;
                        };
                        match message {
                            Message::Call(body) => {
                                let reply = match body.procedure {
                                    ProcedureCall::Null => ProcedureReply::Null,
                                    ProcedureCall::Compound(args) => {
                                        // TODO minorversion
                                        let mut resarray = Vec::with_capacity(args.argarray.len());
                                        for nfs_argop in args.argarray {
                                            resarray.push(match nfs_argop {
                                                NfsArgOp::ExchangeId(args) => NfsResOp::ExchangeId(
                                                    ExchangeIdResult::NFS4_OK(ExchangeIdResultOk {
                                                        client_id: 1.into(),
                                                        sequence_id: 1.into(),
                                                        flags: ExchangeIdFlags::empty(),
                                                        state_protect: StateProtectResult::None,
                                                        server_owner: ServerOwner {
                                                            minor_id: 1234,
                                                            major_id: (&[1, 2, 3, 4]).into(),
                                                        },
                                                        server_scope: vec![].into(),
                                                        server_impl_id: Some(NfsImplId {
                                                            domain: "boman.io".into(),
                                                            name: "lararium".into(),
                                                            date: Time {
                                                                seconds: 0,
                                                                nanoseconds: 0,
                                                            },
                                                        }),
                                                    }),
                                                ),
                                                _ => todo!(),
                                            });
                                        }
                                        ProcedureReply::Compound(CompoundResult {
                                            status: Status::NFS4_OK,
                                            tag: args.tag,
                                            resarray,
                                        })
                                    }
                                };
                                let rpc_msg = RpcMessage {
                                    xid,
                                    message: Message::Reply(Reply::Accepted(AcceptedReply {
                                        verf: OpaqueAuth {
                                            flavor: AuthFlavor::AUTH_NONE, // TODO
                                            body: (&[]).into(),            // TODO
                                        },
                                        body: AcceptedReplyBody::Success(reply),
                                    })),
                                };
                                let mut buffer = [0; 1024];
                                let output = {
                                    let generator = protocol::encode(rpc_msg);
                                    let cursor = Cursor::new(&mut buffer[..]);
                                    let Ok((_, position)) = gen(generator, cursor) else {
                                        tracing::debug!("Failed to encode reply.");
                                        break;
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
                            Message::Reply(body) => {}
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

impl<T> Connection<T> where T: Handler + Clone + Send + Sync + 'static {}

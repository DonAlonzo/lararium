use super::Handler;
use crate::protocol::*;

#[derive(Clone)]
pub struct Connection<T>
where
    T: Handler + Clone + Send + Sync + 'static,
{
    handler: T,
}

impl<T> Connection<T>
where
    T: Handler + Clone + Send + Sync + 'static,
{
    pub fn new(handler: T) -> Self {
        Self { handler }
    }

    pub async fn exchange_id<'a>(
        &self,
        args: ExchangeIdArgs<'a>,
    ) -> ExchangeIdResult<'a> {
        tracing::debug!("EXCHANGE_ID");
        ExchangeIdResult::Ok(ExchangeIdResultOk {
            client_id: 1.into(),
            sequence_id: 1.into(),
            flags: ExchangeIdFlags::USE_PNFS_MDS | ExchangeIdFlags::SUPP_MOVED_REFER,
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
        })
    }

    pub async fn create_session<'a>(
        &self,
        args: CreateSessionArgs<'a>,
    ) -> CreateSessionResult {
        tracing::debug!("CREATE_SESSION");
        CreateSessionResult::Ok(CreateSessionResultOk {
            session_id: [1; 16].into(),
            sequence_id: args.sequence_id,
            flags: CreateSessionFlags::CONN_BACK_CHAN,
            fore_channel_attributes: args.fore_channel_attributes,
            back_channel_attributes: args.back_channel_attributes,
        })
    }

    pub async fn destroy_client_id<'a>(
        &self,
        args: DestroyClientIdArgs,
    ) -> DestroyClientIdResult {
        tracing::debug!("DESTROY_CLIENT_ID");
        DestroyClientIdResult { error: None }
    }
}

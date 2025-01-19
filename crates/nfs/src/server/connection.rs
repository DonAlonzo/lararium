use super::Handler;
use crate::protocol::*;
use num_traits::FromPrimitive;
use tokio::sync::RwLock;

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

    pub fn begin(&self) -> Transaction<T> {
        Transaction {
            handler: &self.handler,
            current_file_handle: RwLock::new(None),
        }
    }
}

pub struct Transaction<'a, T>
where
    T: Handler + Clone + Send + Sync + 'static,
{
    handler: &'a T,
    current_file_handle: RwLock<Option<FileHandle<'a>>>,
}

impl<'a, T> Transaction<'a, T>
where
    T: Handler + Clone + Send + Sync + 'static,
{
    pub async fn access(
        &self,
        flags: AccessFlags,
    ) -> Result<AccessResult, Error> {
        tracing::debug!("ACCESS");
        match *self.current_file_handle.read().await {
            Some(ref file_handle) => self.handler.access(file_handle.clone(), flags).await,
            None => Err(Error::NOENT),
        }
    }

    pub async fn lookup(
        &self,
        name: Component<'a>,
    ) -> Result<(), Error> {
        tracing::debug!("LOOKUP");
        let mut file_handle_guard = self.current_file_handle.write().await;
        let Some(ref file_handle) = *file_handle_guard else {
            return Err(Error::NOENT);
        };
        let file_handle = self.handler.lookup(file_handle.clone(), name).await?;
        *file_handle_guard = Some(file_handle);
        Ok(())
    }

    pub async fn get_attributes(
        &self,
        args: GetAttributesArgs<'_>,
    ) -> Result<FileAttributes, Error> {
        tracing::debug!("GETATTR");
        let mut attributes = vec![];
        for i in 0..(args.attr_request.len() * 32) {
            if (args.attr_request[i / 32] & (1 << (i % 32))) == 0 {
                continue;
            }
            let Some(attribute) = Attribute::from_usize(i) else {
                tracing::debug!(" - N/A: {i}");
                continue;
            };
            attributes.push(attribute);
        }
        match *self.current_file_handle.read().await {
            Some(ref file_handle) => Ok(FileAttributes {
                values: self
                    .handler
                    .get_attributes(file_handle.clone(), &attributes)
                    .await?,
            }),
            None => Err(Error::NOENT),
        }
    }

    pub async fn get_file_handle(&self) -> Result<FileHandle<'a>, Error> {
        tracing::debug!("GETFH");
        match *self.current_file_handle.read().await {
            Some(ref file_handle) => Ok(file_handle.clone()),
            None => Err(Error::NOENT),
        }
    }

    pub async fn put_file_handle(
        &self,
        file_handle: FileHandle<'a>,
    ) -> Result<(), Error> {
        tracing::debug!("PUTFH {file_handle:?}");
        *self.current_file_handle.write().await = Some(file_handle);
        Ok(())
    }

    pub async fn put_root_file_handle(&self) -> Result<(), Error> {
        tracing::debug!("PUTROOTFH");
        *self.current_file_handle.write().await = Some(FileHandle::from(Opaque::from(&[0])));
        Ok(())
    }

    pub async fn read_directory(
        &self,
        args: ReadDirectoryArgs<'a>,
    ) -> Result<ReadDirectoryResult<'a>, Error> {
        tracing::debug!("READDIR");
        match *self.current_file_handle.read().await {
            Some(ref file_handle) => self.handler.read_directory(file_handle.clone(), args).await,
            None => Err(Error::NOENT),
        }
    }

    pub async fn exchange_id<'b>(
        &self,
        args: ExchangeIdArgs<'b>,
    ) -> Result<ExchangeIdResult<'b>, Error> {
        tracing::debug!("EXCHANGE_ID");
        Ok(ExchangeIdResult {
            client_id: 1.into(),
            sequence_id: 1.into(),
            flags: ExchangeIdFlags::USE_PNFS_MDS | ExchangeIdFlags::SUPP_MOVED_REFER,
            state_protect: StateProtectResult::None,
            server_owner: ServerOwner {
                minor_id: 0,
                major_id: (&[0; 16]).into(),
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

    pub async fn create_session(
        &self,
        args: CreateSessionArgs<'_>,
    ) -> Result<CreateSessionResult, Error> {
        tracing::debug!("CREATE_SESSION");
        Ok(CreateSessionResult {
            session_id: [1; 16].into(),
            sequence_id: args.sequence_id,
            flags: CreateSessionFlags::CONN_BACK_CHAN,
            fore_channel_attributes: args.fore_channel_attributes,
            back_channel_attributes: args.back_channel_attributes,
        })
    }

    pub async fn destroy_session(
        &self,
        args: DestroySessionArgs,
    ) -> Result<(), Error> {
        tracing::debug!("DESTROY_SESSION");
        Ok(())
    }

    pub async fn destroy_client_id(
        &self,
        args: DestroyClientIdArgs,
    ) -> Result<(), Error> {
        tracing::debug!("DESTROY_CLIENT_ID");
        Ok(())
    }

    pub async fn get_security_info(
        &self,
        args: GetSecurityInfoArgs<'_>,
    ) -> GetSecurityInfoResult {
        tracing::debug!("SECINFO");
        GetSecurityInfoResult::Ok(GetSecurityInfoResultOk(vec![GetSecurityInfo::AuthNone]))
    }

    pub async fn get_security_info_no_name(
        &self,
        args: GetSecurityInfoNoNameArgs,
    ) -> GetSecurityInfoNoNameResult {
        tracing::debug!("SECINFO_NO_NAME");
        GetSecurityInfoNoNameResult(GetSecurityInfoResult::Ok(GetSecurityInfoResultOk(vec![
            GetSecurityInfo::AuthNone,
        ])))
    }

    pub async fn sequence(
        &self,
        args: SequenceArgs,
    ) -> Result<SequenceResult, Error> {
        tracing::debug!("SEQUENCE");
        Ok(SequenceResult {
            session_id: [1; 16].into(),
            sequence_id: args.sequence_id,
            slot_id: args.slot_id,
            highest_slot_id: args.highest_slot_id,
            target_highest_slot_id: args.highest_slot_id,
            status_flags: SequenceStatusFlags::empty(),
        })
    }

    pub async fn reclaim_complete(
        &self,
        args: ReclaimCompleteArgs,
    ) -> Result<(), Error> {
        tracing::debug!("RECLAIM_COMPLETE");
        Ok(())
    }
}

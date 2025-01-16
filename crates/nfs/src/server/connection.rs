use super::Handler;
use crate::protocol::*;
use num_traits::FromPrimitive;

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

    pub async fn get_attributes(
        &self,
        args: GetAttributesArgs<'_>,
    ) -> GetAttributesResult {
        tracing::debug!("GETATTR");
        for i in 0..(args.attr_request.len() * 32) {
            if (args.attr_request[i / 32] & (1 << (i % 32))) == 0 {
                continue;
            }
            let Some(attribute) = Attribute::from_usize(i) else {
                continue;
            };
            match attribute {
                Attribute::SUPPORTED_ATTRS => println!("supported_attrs"),
                Attribute::TYPE => println!("type"),
                Attribute::FH_EXPIRE_TYPE => println!("fh_expire_type"),
                Attribute::CHANGE => println!("change"),
                Attribute::SIZE => println!("size"),
                Attribute::LINK_SUPPORT => println!("link_support"),
                Attribute::SYMLINK_SUPPORT => println!("symlink_support"),
                Attribute::NAMED_ATTR => println!("named_attr"),
                Attribute::FSID => println!("fsid"),
                Attribute::UNIQUE_HANDLES => println!("unique_handles"),
                Attribute::LEASE_TIME => println!("lease_time"),
                Attribute::RDATTR_ERROR => println!("rdattr_error"),
                Attribute::FILEHANDLE => println!("filehandle"),
                Attribute::SUPPATTR_EXCLCREAT => println!("suppattr_exclcreat"),
            }
        }
        GetAttributesResult::Ok(GetAttributesResultOk {
            obj_attributes: FileAttributes {
                mask: vec![].into(),
                values: vec![].into(),
            },
        })
    }

    pub async fn get_file_handle(&self) -> GetFileHandleResult {
        tracing::debug!("GETFH");
        GetFileHandleResult::Ok(GetFileHandleResultOk {
            object: FileHandle::from(Opaque::from(&[2; 128])),
        })
    }

    pub async fn put_file_handle(
        &self,
        args: PutFileHandleArgs<'_>,
    ) -> PutFileHandleResult {
        tracing::debug!("PUTFH");
        PutFileHandleResult { error: None }
    }

    pub async fn put_root_file_handle(&self) -> PutRootFileHandleResult {
        tracing::debug!("PUTROOTFH");
        PutRootFileHandleResult { error: None }
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

    pub async fn destroy_session(
        &self,
        args: DestroySessionArgs,
    ) -> DestroySessionResult {
        tracing::debug!("DESTROY_SESSION");
        DestroySessionResult { error: None }
    }

    pub async fn destroy_client_id<'a>(
        &self,
        args: DestroyClientIdArgs,
    ) -> DestroyClientIdResult {
        tracing::debug!("DESTROY_CLIENT_ID");
        DestroyClientIdResult { error: None }
    }

    pub async fn get_security_info(
        &self,
        args: GetSecurityInfoArgs<'_>,
    ) -> GetSecurityInfoResult {
        tracing::debug!("SECINFO");
        todo!()
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
    ) -> SequenceResult {
        tracing::debug!("SEQUENCE");
        SequenceResult::Ok(SequenceResultOk {
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
    ) -> ReclaimCompleteResult {
        tracing::debug!("RECLAIM_COMPLETE");
        ReclaimCompleteResult { error: None }
    }
}

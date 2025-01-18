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

    pub async fn access(
        &self,
        flags: AccessFlags,
    ) -> Result<AccessResult, Error> {
        Ok(AccessResult {
            supported: AccessFlags::READ | AccessFlags::LOOKUP,
            access: AccessFlags::READ | AccessFlags::LOOKUP,
        })
    }

    pub async fn lookup(
        &self,
        name: Component<'_>,
    ) -> Result<(), Error> {
        tracing::debug!("LOOKUP");
        Ok(())
    }

    pub async fn get_attributes(
        &self,
        args: GetAttributesArgs<'_>,
    ) -> Result<FileAttributes, Error> {
        tracing::debug!("GETATTR");
        let mut values = vec![];
        for i in 0..(args.attr_request.len() * 32) {
            if (args.attr_request[i / 32] & (1 << (i % 32))) == 0 {
                continue;
            }
            let Some(attribute) = Attribute::from_usize(i) else {
                tracing::debug!(" - N/A: {i}");
                continue;
            };
            tracing::debug!(" - {attribute:?}");
            values.push(match attribute {
                Attribute::SupportedAttributes => AttributeValue::SupportedAttributes(
                    vec![
                        Attribute::SupportedAttributes,
                        Attribute::Type,
                        Attribute::FileHandleExpireType,
                        Attribute::Change,
                        Attribute::Size,
                        Attribute::LinkSupport,
                        Attribute::SymlinkSupport,
                        Attribute::NamedAttributes,
                        Attribute::FileSystemId,
                        Attribute::UniqueHandles,
                        Attribute::LeaseTime,
                        Attribute::ReadDirAttributeError,
                        Attribute::FileHandle,
                        Attribute::FileId,
                        Attribute::MaxFileSize,
                        Attribute::MaxRead,
                        Attribute::MaxWrite,
                        Attribute::Mode,
                        Attribute::SupportedAttributesExclusiveCreate,
                    ]
                    .into(),
                ),
                Attribute::Type => AttributeValue::Type(FileType::Directory),
                Attribute::FileHandleExpireType => AttributeValue::FileHandleExpireType(0),
                Attribute::Change => AttributeValue::Change(5),
                Attribute::Size => AttributeValue::Size(1337),
                Attribute::LinkSupport => AttributeValue::LinkSupport(false),
                Attribute::SymlinkSupport => AttributeValue::SymlinkSupport(false),
                Attribute::NamedAttributes => AttributeValue::NamedAttributes(false),
                Attribute::FileSystemId => {
                    AttributeValue::FileSystemId(FileSystemId { major: 0, minor: 0 })
                }
                Attribute::UniqueHandles => AttributeValue::UniqueHandles(true),
                Attribute::LeaseTime => AttributeValue::LeaseTime(90),
                Attribute::ReadDirAttributeError => AttributeValue::ReadDirAttributeError,
                Attribute::AclSupport => AttributeValue::AclSupport(AclSupportFlags::empty()),
                Attribute::CaseInsensitive => AttributeValue::CaseInsensitive(false),
                Attribute::CasePreserving => AttributeValue::CasePreserving(true),
                Attribute::FileHandle => {
                    AttributeValue::FileHandle(FileHandle::from(Opaque::from(&[1, 2, 3, 4])))
                }
                Attribute::FileId => AttributeValue::FileId(42000),
                Attribute::MaxFileSize => AttributeValue::MaxFileSize(1024 * 1024 * 1024 * 1024),
                Attribute::MaxRead => AttributeValue::MaxRead(1024 * 1024),
                Attribute::MaxWrite => AttributeValue::MaxWrite(1024 * 1024),
                Attribute::Mode => AttributeValue::Mode(0xFFF.into()),
                Attribute::NumberOfLinks => AttributeValue::NumberOfLinks(0),
                Attribute::MountedOnFileId => AttributeValue::MountedOnFileId(42001),
                Attribute::SupportedAttributesExclusiveCreate => {
                    AttributeValue::SupportedAttributesExclusiveCreate(
                        vec![
                            Attribute::SupportedAttributes,
                            Attribute::Type,
                            Attribute::FileHandleExpireType,
                            Attribute::Change,
                            Attribute::Size,
                            Attribute::LinkSupport,
                            Attribute::SymlinkSupport,
                            Attribute::NamedAttributes,
                            Attribute::FileSystemId,
                            Attribute::UniqueHandles,
                            Attribute::LeaseTime,
                            Attribute::ReadDirAttributeError,
                            Attribute::FileHandle,
                            Attribute::FileId,
                            Attribute::MaxFileSize,
                            Attribute::MaxRead,
                            Attribute::MaxWrite,
                            Attribute::Mode,
                            Attribute::SupportedAttributesExclusiveCreate,
                        ]
                        .into(),
                    )
                }
            });
        }
        Ok(FileAttributes { values })
    }

    pub async fn get_file_handle(&self) -> Result<FileHandle, Error> {
        tracing::debug!("GETFH");
        Ok(FileHandle::from(Opaque::from(&[1, 2, 3, 4])))
    }

    pub async fn put_file_handle(
        &self,
        args: PutFileHandleArgs<'_>,
    ) -> Result<(), Error> {
        tracing::debug!("PUTFH");
        Ok(())
    }

    pub async fn put_root_file_handle(&self) -> Result<(), Error> {
        tracing::debug!("PUTROOTFH");
        Ok(())
    }

    pub async fn exchange_id<'a>(
        &self,
        args: ExchangeIdArgs<'a>,
    ) -> Result<ExchangeIdResult<'a>, Error> {
        tracing::debug!("EXCHANGE_ID");
        Ok(ExchangeIdResult {
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

    pub async fn destroy_client_id<'a>(
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

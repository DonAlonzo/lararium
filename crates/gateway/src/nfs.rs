use lararium_nfs::*;

impl Handler for crate::Gateway {
    async fn access(
        &self,
        file_handle: FileHandle<'_>,
        flags: AccessFlags,
    ) -> Result<AccessResult, Error> {
        Ok(AccessResult {
            supported: AccessFlags::READ | AccessFlags::LOOKUP | AccessFlags::EXECUTE,
            access: AccessFlags::READ | AccessFlags::LOOKUP | AccessFlags::EXECUTE,
        })
    }

    async fn lookup<'a>(
        &self,
        file_handle: FileHandle<'a>,
        name: Component<'a>,
    ) -> Result<FileHandle<'a>, Error> {
        Ok(FileHandle::from(name.as_bytes().to_vec()))
    }

    async fn get_attributes<'a>(
        &self,
        file_handle: FileHandle<'a>,
        mask: AttributeMask<'a>,
    ) -> Result<Vec<AttributeValue<'a>>, Error> {
        let mut values = Vec::new();
        for attribute in mask.into_iter() {
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
                    AttributeValue::FileHandle(FileHandle::from(&[1, 2, 3, 4]))
                }
                Attribute::FileId => AttributeValue::FileId(42000),
                Attribute::MaxFileSize => AttributeValue::MaxFileSize(1024 * 1024 * 1024 * 1024),
                Attribute::MaxRead => AttributeValue::MaxRead(1024 * 1024),
                Attribute::MaxWrite => AttributeValue::MaxWrite(1024 * 1024),
                Attribute::Mode => AttributeValue::Mode(0o0777.into()),
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
        Ok(values)
    }

    async fn read_directory<'a>(
        &self,
        file_handle: FileHandle<'a>,
        args: ReadDirectoryArgs<'a>,
    ) -> Result<ReadDirectoryResult<'a>, Error> {
        for attribute in args.attributes.into_iter() {
            tracing::debug!(" - {attribute:?}");
        }
        Ok(ReadDirectoryResult {
            cookie_verf: Verifier::from([0, 1, 2, 3, 4, 5, 6, 7]),
            directory_list: DirectoryList {
                entries: vec![Entry {
                    cookie: 0,
                    name: "hello world".into(),
                    attributes: vec![],
                }],
                eof: true,
            },
        })
    }
}

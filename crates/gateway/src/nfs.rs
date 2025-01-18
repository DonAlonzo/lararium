use lararium_nfs::*;

impl Handler for crate::Gateway {
    async fn access(
        &self,
        file_handle: FileHandle<'_>,
        flags: AccessFlags,
    ) -> Result<AccessResult, Error> {
        Ok(AccessResult {
            supported: AccessFlags::READ | AccessFlags::LOOKUP,
            access: AccessFlags::READ | AccessFlags::LOOKUP,
        })
    }

    async fn lookup<'a>(
        &self,
        file_handle: FileHandle<'a>,
        name: Component<'a>,
    ) -> Result<FileHandle<'a>, Error> {
        Ok(FileHandle::from(Opaque::from(name.as_bytes().to_vec())))
    }

    async fn get_attributes<'a, 'b>(
        &self,
        file_handle: FileHandle<'a>,
        attributes: &'b [Attribute],
    ) -> Result<Vec<AttributeValue<'a>>, Error> {
        let mut values = Vec::new();
        for attribute in attributes {
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
        Ok(values)
    }
}

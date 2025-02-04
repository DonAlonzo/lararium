use super::*;

use cookie_factory::{
    bytes::{be_i64, be_u32, be_u64, be_u8},
    combinator::{back_to_the_buffer, slice},
    gen, gen_simple,
    multi::many_ref,
    sequence::tuple,
    Seek, SerializeFn,
};
use std::io::Write;
use std::iter::repeat;

#[inline(always)]
fn bool_u32<W: Write>(value: bool) -> impl SerializeFn<W> {
    be_u32(value as u32)
}

#[inline(always)]
fn opaque<'a, W: Write + 'a>(value: &'a [u8]) -> impl SerializeFn<W> + 'a {
    let alignment = (4 - (value.len() as usize % 4)) % 4;
    tuple((slice(value), many_ref(repeat(0u8).take(alignment), be_u8)))
}

#[inline(always)]
fn variable_length_opaque<'a, W: Write + 'a>(value: &'a [u8]) -> impl SerializeFn<W> + 'a {
    tuple((be_u32(value.len() as u32), opaque(value)))
}

#[inline(always)]
fn string<'a, W: Write + 'a>(value: &'a str) -> impl SerializeFn<W> + 'a {
    variable_length_opaque(value.as_bytes())
}

#[inline(always)]
fn variable_length_array<E, It, I, F, G, W: Write>(
    items: I,
    generator: F,
) -> impl SerializeFn<W>
where
    It: Iterator<Item = E> + Clone,
    I: IntoIterator<Item = E, IntoIter = It>,
    F: Fn(E) -> G,
    G: SerializeFn<W>,
{
    let items = items.into_iter();
    tuple((
        be_u32(items.clone().count() as u32),
        many_ref(items, generator),
    ))
}

#[inline(always)]
fn bitmap<'a, 'b: 'a, W: Write + 'a>(value: &'a Bitmap<'b>) -> impl SerializeFn<W> + 'a {
    variable_length_array(&*value.0, |x| be_u32(*x))
}

#[inline(always)]
fn nfs_opnum<W: Write>(value: NfsOpnum) -> impl SerializeFn<W> {
    be_u32(value as u32)
}

#[inline(always)]
fn file_type<W: Write>(value: FileType) -> impl SerializeFn<W> {
    be_u32(value as u32)
}

#[inline(always)]
fn file_system_id<'a, W: Write + 'a>(value: &'a FileSystemId) -> impl SerializeFn<W> + 'a {
    tuple((be_u64(value.major), be_u64(value.minor)))
}

#[inline(always)]
fn error<W: Write>(value: Option<Error>) -> impl SerializeFn<W> {
    be_u32(match value {
        None => 0,
        Some(code) => code as u32,
    })
}

#[inline(always)]
fn time<W: Write>(value: Time) -> impl SerializeFn<W> {
    tuple((be_i64(value.seconds), be_u32(value.nanoseconds)))
}

#[inline(always)]
fn file_attributes<'a, 'b: 'a, W: Write + Seek + 'a>(
    values: &'a [AttributeValue<'b>]
) -> impl SerializeFn<W> + 'a {
    tuple((
        attribute_mask(values.iter().map(|v| v.attribute())),
        back_to_the_buffer(
            4,
            move |out| gen(many_ref(values, attribute_value), out),
            move |out, length| gen_simple(be_u32(length as u32), out),
        ),
    ))
}

#[inline(always)]
fn attribute_mask<'a, T, W>(attributes: T) -> impl SerializeFn<W> + 'a
where
    T: IntoIterator<Item = Attribute> + Clone + 'a,
    W: Write + 'a,
{
    move |out| {
        let Some(max) = attributes.clone().into_iter().map(|a| a as usize).max() else {
            return be_u32(0)(out);
        };
        let bitmap_size = max / 32 + 1;
        let mut bitmap = vec![0; bitmap_size];
        for attribute in attributes.clone().into_iter() {
            let attribute = attribute as usize;
            let word_index = attribute / 32;
            let bit_index = attribute % 32;
            bitmap[word_index] |= 1 << bit_index;
        }
        variable_length_array(bitmap, be_u32)(out)
    }
}

#[inline(always)]
fn attribute_value<'a, 'b: 'a, W: Write + 'a>(
    value: &'a AttributeValue<'b>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        AttributeValue::SupportedAttributes(value) => {
            attribute_mask(value.into_iter().cloned())(out)
        }
        AttributeValue::Type(value) => file_type(*value)(out),
        AttributeValue::FileHandleExpireType(value) => be_u32(*value)(out),
        AttributeValue::Change(value) => be_u64(*value)(out),
        AttributeValue::Size(value) => be_u64(*value)(out),
        AttributeValue::LinkSupport(value) => bool_u32(*value)(out),
        AttributeValue::SymlinkSupport(value) => bool_u32(*value)(out),
        AttributeValue::NamedAttributes(value) => bool_u32(*value)(out),
        AttributeValue::FileSystemId(value) => file_system_id(value)(out),
        AttributeValue::UniqueHandles(value) => bool_u32(*value)(out),
        AttributeValue::LeaseTime(value) => be_u32(*value)(out),
        AttributeValue::ReadDirAttributeError => todo!(),
        AttributeValue::AclSupport(value) => acl_support_flags(*value)(out),
        AttributeValue::CaseInsensitive(value) => bool_u32(*value)(out),
        AttributeValue::CasePreserving(value) => bool_u32(*value)(out),
        AttributeValue::FileHandle(value) => file_handle(value)(out),
        AttributeValue::FileId(value) => be_u64(*value)(out),
        AttributeValue::MaxFileSize(value) => be_u64(*value)(out),
        AttributeValue::MaxRead(value) => be_u64(*value)(out),
        AttributeValue::MaxWrite(value) => be_u64(*value)(out),
        AttributeValue::Mode(value) => be_u32(*value)(out),
        AttributeValue::NumberOfLinks(value) => be_u32(*value)(out),
        AttributeValue::MountedOnFileId(value) => be_u64(*value)(out),
        AttributeValue::SupportedAttributesExclusiveCreate(value) => {
            attribute_mask(value.into_iter().cloned())(out)
        }
    }
}

#[inline(always)]
fn nfs_impl_id<'a, 'b: 'a, W: Write + 'a>(value: &'a NfsImplId<'b>) -> impl SerializeFn<W> + 'a {
    tuple((string(&value.domain), string(&value.name), time(value.date)))
}

#[inline(always)]
fn gss_handle<'a, 'b: 'a, W: Write + 'a>(value: &'a GssHandle<'b>) -> impl SerializeFn<W> + 'a {
    variable_length_opaque(&value.0)
}

#[inline(always)]
fn sec_oid<'a, 'b: 'a, W: Write + 'a>(value: &'a SecOid<'b>) -> impl SerializeFn<W> + 'a {
    variable_length_opaque(&value.0)
}

#[inline(always)]
fn file_handle<'a, 'b: 'a, W: Write + 'a>(value: &'a FileHandle<'b>) -> impl SerializeFn<W> + 'a {
    tuple((be_u32(value.0.len() as u32), opaque(value)))
}

#[inline(always)]
fn server_owner<'a, 'b: 'a, W: Write + 'a>(value: &'a ServerOwner<'b>) -> impl SerializeFn<W> + 'a {
    tuple((
        be_u64(value.minor_id),
        variable_length_opaque(&value.major_id),
    ))
}

#[inline(always)]
fn client_owner<'a, 'b: 'a, W: Write + 'a>(value: &'a ClientOwner<'b>) -> impl SerializeFn<W> + 'a {
    tuple((
        slice(&value.verifier),
        variable_length_opaque(&value.owner_id),
    ))
}

#[inline(always)]
fn change_info<'a, W: Write + 'a>(value: &'a ChangeInfo) -> impl SerializeFn<W> + 'a {
    tuple((
        bool_u32(value.atomic),
        be_u64(value.before),
        be_u64(value.after),
    ))
}

#[inline(always)]
fn state_owner<'a, 'b: 'a, W: Write + 'a>(value: &'a StateOwner<'b>) -> impl SerializeFn<W> + 'a {
    tuple((
        be_u64(value.client_id),
        variable_length_opaque(&value.owner),
    ))
}

#[inline(always)]
fn open_owner<'a, 'b: 'a, W: Write + 'a>(value: &'a OpenOwner<'b>) -> impl SerializeFn<W> + 'a {
    state_owner(&value.0)
}

#[inline(always)]
fn state_id<'a, W: Write + 'a>(value: &'a StateId) -> impl SerializeFn<W> + 'a {
    tuple((be_u32(value.sequence_id), slice(value.other)))
}

#[inline(always)]
fn acl_support_flags<W: Write>(flags: AclSupportFlags) -> impl SerializeFn<W> {
    be_u32(flags.bits() as u32)
}

#[inline(always)]
fn state_protect_ops<'a, 'b: 'a, W: Write + 'a>(
    value: &'a StateProtectOps<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((bitmap(&value.must_enforce), bitmap(&value.must_allow)))
}

#[inline(always)]
fn ssv_sp_parms<'a, 'b: 'a, W: Write + 'a>(value: &'a SsvSpParms<'b>) -> impl SerializeFn<W> + 'a {
    tuple((
        state_protect_ops(&value.ops),
        variable_length_array(&value.hash_algs, sec_oid),
        variable_length_array(&value.encr_algs, sec_oid),
        be_u32(value.window),
        be_u32(value.num_gss_handles),
    ))
}

#[inline(always)]
fn state_protect_how<W: Write>(value: StateProtectHow) -> impl SerializeFn<W> {
    be_u32(value as u32)
}

#[inline(always)]
fn state_protect_args<'a, 'b: 'a, W: Write + 'a>(
    value: &'a StateProtectArgs<'b>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        StateProtectArgs::None => state_protect_how(StateProtectHow::None)(out),
        StateProtectArgs::MachineCredentials(ref mach_ops) => tuple((
            state_protect_how(StateProtectHow::MachineCredentials),
            state_protect_ops(mach_ops),
        ))(out),
        StateProtectArgs::ServerSideValidation(ref ssv_parms) => tuple((
            state_protect_how(StateProtectHow::ServerSideValidation),
            ssv_sp_parms(ssv_parms),
        ))(out),
    }
}

#[inline(always)]
fn state_protect_result<'a, 'b: 'a, W: Write + 'a>(
    value: &'a StateProtectResult<'b>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        StateProtectResult::None => state_protect_how(StateProtectHow::None)(out),
        StateProtectResult::MachineCredentials(mach_ops) => tuple((
            state_protect_how(StateProtectHow::MachineCredentials),
            state_protect_ops(mach_ops),
        ))(out),
        StateProtectResult::ServerSideValidation(ssv_info) => tuple((
            state_protect_how(StateProtectHow::ServerSideValidation),
            ssv_prot_info(ssv_info),
        ))(out),
    }
}

#[inline(always)]
fn ssv_prot_info<'a, 'b: 'a, W: Write + 'a>(
    value: &'a SsvProtInfo<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((
        state_protect_ops(&value.ops),
        be_u32(value.hash_alg),
        be_u32(value.encr_alg),
        be_u32(value.ssv_len),
        be_u32(value.window),
        variable_length_array(&value.handles, gss_handle),
    ))
}

#[inline(always)]
fn nfs_resop<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'a NfsResOp<'b>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        NfsResOp::Access(ref value) => {
            tuple((nfs_opnum(NfsOpnum::Access), access_result(value)))(out)
        }
        NfsResOp::Close(ref value) => tuple((nfs_opnum(NfsOpnum::Close), close_result(value)))(out),
        NfsResOp::GetAttributes(ref value) => tuple((
            nfs_opnum(NfsOpnum::GetAttributes),
            get_attributes_result(value),
        ))(out),
        NfsResOp::GetFileHandle(ref value) => tuple((
            nfs_opnum(NfsOpnum::GetFileHandle),
            get_file_handle_result(value),
        ))(out),
        NfsResOp::Lookup(ref value) => {
            tuple((nfs_opnum(NfsOpnum::Lookup), lookup_result(value)))(out)
        }
        NfsResOp::Open(ref value) => tuple((nfs_opnum(NfsOpnum::Open), open_result(value)))(out),
        NfsResOp::PutFileHandle(ref value) => tuple((
            nfs_opnum(NfsOpnum::PutFileHandle),
            put_file_handle_result(value),
        ))(out),
        NfsResOp::PutRootFileHandle(ref value) => tuple((
            nfs_opnum(NfsOpnum::PutRootFileHandle),
            put_root_file_handle_result(value),
        ))(out),
        NfsResOp::Read(ref value) => tuple((nfs_opnum(NfsOpnum::Read), read_result(value)))(out),
        NfsResOp::ReadDirectory(ref value) => tuple((
            nfs_opnum(NfsOpnum::ReadDirectory),
            read_directory_result(value),
        ))(out),
        NfsResOp::GetSecurityInfo(ref value) => tuple((
            nfs_opnum(NfsOpnum::GetSecurityInfo),
            get_security_info_result(value),
        ))(out),
        NfsResOp::ExchangeId(ref value) => {
            tuple((nfs_opnum(NfsOpnum::ExchangeId), exchange_id_result(value)))(out)
        }
        NfsResOp::CreateSession(ref value) => tuple((
            nfs_opnum(NfsOpnum::CreateSession),
            create_session_result(value),
        ))(out),
        NfsResOp::DestroySession(ref value) => tuple((
            nfs_opnum(NfsOpnum::DestroySession),
            destroy_session_result(value),
        ))(out),
        NfsResOp::DestroyClientId(ref value) => tuple((
            nfs_opnum(NfsOpnum::DestroyClientId),
            destroy_client_id_result(value),
        ))(out),
        NfsResOp::GetSecurityInfoNoName(ref value) => tuple((
            nfs_opnum(NfsOpnum::GetSecurityInfoNoName),
            get_security_info_no_name_result(value),
        ))(out),
        NfsResOp::Sequence(ref value) => {
            tuple((nfs_opnum(NfsOpnum::Sequence), sequence_result(value)))(out)
        }
        NfsResOp::ReclaimComplete(ref value) => tuple((
            nfs_opnum(NfsOpnum::ReclaimComplete),
            reclaim_complete_result(value),
        ))(out),
    }
}

#[inline(always)]
fn compound_result<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'a CompoundResult<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((
        error(value.error),
        string(&value.tag),
        variable_length_array(&value.resarray, nfs_resop),
    ))
}

// Attribute 12: acl

#[inline(always)]
fn ace_type<W: Write>(flags: AceType) -> impl SerializeFn<W> {
    be_u32(flags.bits() as u32)
}

#[inline(always)]
fn ace_flag<W: Write>(flags: AceFlag) -> impl SerializeFn<W> {
    be_u32(flags.bits() as u32)
}

#[inline(always)]
fn ace_access_mask<W: Write>(flags: AceAccessMask) -> impl SerializeFn<W> {
    be_u32(flags.bits() as u32)
}

#[inline(always)]
fn nfs_ace<'a, 'b: 'a, W: Write + 'a>(value: &'a NfsAce<'b>) -> impl SerializeFn<W> + 'a {
    tuple((
        ace_type(value.r#type),
        ace_flag(value.flag),
        ace_access_mask(value.access_mask),
        string(&value.who),
    ))
}

// Operation 3: ACCESS

#[inline(always)]
fn access_flags<W: Write>(flags: AccessFlags) -> impl SerializeFn<W> {
    be_u32(flags.bits() as u32)
}

#[inline(always)]
fn access_result<'a, W: Write + Seek + 'a>(
    value: &'a Result<AccessResult, Error>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(ref value) => tuple((error(None), access_result_ok(value)))(out),
        Err(value) => error(Some(*value))(out),
    }
}

#[inline(always)]
fn access_result_ok<'a, W: Write + 'a>(value: &'a AccessResult) -> impl SerializeFn<W> + 'a {
    tuple((access_flags(value.supported), access_flags(value.access)))
}

// Operation 4: CLOSE

#[inline(always)]
fn close_args<'a, W: Write + 'a>(value: &'a CloseArgs) -> impl SerializeFn<W> + 'a {
    tuple((be_u32(value.sequence_id), state_id(&value.open_state_id)))
}

#[inline(always)]
fn close_result<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'a Result<StateId, Error>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(ref value) => tuple((error(None), state_id(value)))(out),
        Err(value) => error(Some(*value))(out),
    }
}

// Operation 9: GETATTR

#[inline(always)]
fn get_attributes_result<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'a Result<Vec<AttributeValue<'b>>, Error>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(ref value) => tuple((error(None), file_attributes(value)))(out),
        Err(value) => error(Some(*value))(out),
    }
}

// Operation 10: GETFH

#[inline(always)]
fn get_file_handle_result<'a, 'b: 'a, W: Write + 'a>(
    value: &'a Result<FileHandle<'b>, Error>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(ref value) => tuple((error(None), file_handle(value)))(out),
        Err(value) => error(Some(*value))(out),
    }
}

// Operation 15: LOOKUP

#[inline(always)]
fn lookup_result<'a, W: Write + 'a>(value: &'a Result<(), Error>) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(_) => error(None)(out),
        Err(value) => error(Some(*value))(out),
    }
}

// Operation 18: OPEN

#[inline(always)]
fn space_limit_discriminant<W: Write>(value: SpaceLimitDiscriminant) -> impl SerializeFn<W> {
    be_u32(value as u32)
}

#[inline(always)]
fn nfs_modified_limit<'a, W: Write + 'a>(value: &'a NfsModifiedLimit) -> impl SerializeFn<W> + 'a {
    tuple((be_u32(value.num_blocks), be_u32(value.bytes_per_block)))
}

#[inline(always)]
fn space_limit<'a, W: Write + 'a>(value: &'a SpaceLimit) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        SpaceLimit::Size(size) => tuple((
            space_limit_discriminant(SpaceLimitDiscriminant::Size),
            be_u64(*size),
        ))(out),
        SpaceLimit::Blocks(limit) => tuple((
            space_limit_discriminant(SpaceLimitDiscriminant::Blocks),
            nfs_modified_limit(limit),
        ))(out),
    }
}

#[inline(always)]
fn open_delegation_type<W: Write>(value: OpenDelegationType) -> impl SerializeFn<W> {
    be_u32(value as u32)
}

#[inline(always)]
fn open_read_delegation<'a, 'b: 'a, W: Write + 'a>(
    value: &'a OpenReadDelegation<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((
        state_id(&value.state_id),
        bool_u32(value.recall),
        nfs_ace(&value.permissions),
    ))
}

#[inline(always)]
fn open_write_delegation<'a, 'b: 'a, W: Write + 'a>(
    value: &'a OpenWriteDelegation<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((
        state_id(&value.state_id),
        bool_u32(value.recall),
        space_limit(&value.space_limit),
        nfs_ace(&value.permissions),
    ))
}

#[inline(always)]
fn open_none_delegation_discriminant<W: Write>(
    value: OpenNoneDelegationDiscriminant
) -> impl SerializeFn<W> {
    be_u32(value as u32)
}

#[inline(always)]
fn open_none_delegation<'a, W: Write + 'a>(
    value: &'a OpenNoneDelegation
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        _ => todo!(),
    }
}

#[inline(always)]
fn open_delegation<'a, 'b: 'a, W: Write + 'a>(
    value: &'a OpenDelegation<'b>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        OpenDelegation::None => open_delegation_type(OpenDelegationType::None)(out),
        OpenDelegation::Read(value) => tuple((
            open_delegation_type(OpenDelegationType::Read),
            open_read_delegation(value),
        ))(out),
        OpenDelegation::Write(value) => tuple((
            open_delegation_type(OpenDelegationType::Write),
            open_write_delegation(value),
        ))(out),
        OpenDelegation::NoneExt(value) => tuple((
            open_delegation_type(OpenDelegationType::NoneExt),
            open_none_delegation(value),
        ))(out),
    }
}

#[inline(always)]
fn open_result_flags<W: Write>(flags: OpenResultFlags) -> impl SerializeFn<W> {
    be_u32(flags.bits() as u32)
}

#[inline(always)]
fn open_result<'a, 'b: 'a, W: Write + 'a>(
    value: &'a Result<OpenResult<'b>, Error>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(ref value) => tuple((error(None), open_result_ok(value)))(out),
        Err(value) => error(Some(*value))(out),
    }
}

#[inline(always)]
fn open_result_ok<'a, 'b: 'a, W: Write + 'a>(
    value: &'a OpenResult<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((
        state_id(&value.state_id),
        change_info(&value.change_info),
        open_result_flags(value.flags),
        attribute_mask(value.attributes.clone()),
        open_delegation(&value.delegation),
    ))
}

// Operation 22: PUTFH

#[inline(always)]
fn put_file_handle_result<'a, W: Write + 'a>(
    value: &'a Result<(), Error>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(_) => error(None)(out),
        Err(value) => error(Some(*value))(out),
    }
}

// Operation 24: PUTROOTFS

#[inline(always)]
fn put_root_file_handle_result<'a, W: Write + 'a>(
    value: &'a Result<(), Error>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(_) => error(None)(out),
        Err(value) => error(Some(*value))(out),
    }
}

// Operation 25: READ

#[inline(always)]
fn read_args<'a, W: Write + 'a>(value: &'a ReadArgs) -> impl SerializeFn<W> + 'a {
    tuple((
        state_id(&value.state_id),
        be_u64(value.offset),
        be_u32(value.count),
    ))
}

#[inline(always)]
fn read_result<'a, 'b: 'a, W: Write + 'a>(
    value: &'a Result<ReadResult<'b>, Error>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(ref value) => tuple((error(None), read_result_ok(value)))(out),
        Err(value) => error(Some(*value))(out),
    }
}

#[inline(always)]
fn read_result_ok<'a, 'b: 'a, W: Write + 'a>(
    value: &'a ReadResult<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((bool_u32(value.eof), variable_length_opaque(&value.data)))
}

// Operation 26: READDIR

#[inline(always)]
fn entry<'a, 'b: 'a, W: Write + Seek + 'a>(value: &'a Entry<'b>) -> impl SerializeFn<W> + 'a {
    tuple((
        be_u64(value.cookie),
        string(&value.name),
        file_attributes(&value.attributes),
    ))
}

#[inline(always)]
fn directory_list<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'a DirectoryList<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((
        many_ref(value.entries.iter(), |value| {
            tuple((bool_u32(true), entry(value)))
        }),
        bool_u32(false),
        bool_u32(value.eof),
    ))
}

#[inline(always)]
fn read_directory_result<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'a Result<ReadDirectoryResult<'b>, Error>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(ref value) => tuple((error(None), read_directory_result_ok(value)))(out),
        Err(value) => error(Some(*value))(out),
    }
}

#[inline(always)]
fn read_directory_result_ok<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'a ReadDirectoryResult<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((
        slice(&value.cookie_verf),
        directory_list(&value.directory_list),
    ))
}

// Operation 33: SECINFO

#[inline(always)]
fn get_security_info_args<'a, 'b: 'a, W: Write + 'a>(
    value: &'a GetSecurityInfoArgs<'b>
) -> impl SerializeFn<W> + 'a {
    string(&value.name)
}

#[inline(always)]
fn rpc_gss_svc<W: Write>(value: RpcGssSvc) -> impl SerializeFn<W> {
    be_u32(value as u32)
}

#[inline(always)]
fn rpc_sec_gss_info<'a, 'b: 'a, W: Write + 'a>(
    value: &'a RpcSecGssInfo<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((
        sec_oid(&value.oid),
        be_u32(value.qop),
        rpc_gss_svc(value.service),
    ))
}

#[inline(always)]
fn get_security_info<'a, 'b: 'a, W: Write + 'a>(
    value: &'a GetSecurityInfo<'b>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        GetSecurityInfo::RpcSecGss(ref value) => {
            tuple((auth_flavor(AuthFlavor::RpcSecGss), rpc_sec_gss_info(value)))(out)
        }
        GetSecurityInfo::AuthNone => auth_flavor(AuthFlavor::AuthNone)(out),
        GetSecurityInfo::AuthSys => auth_flavor(AuthFlavor::AuthSys)(out),
        GetSecurityInfo::AuthShort => auth_flavor(AuthFlavor::AuthShort)(out),
        GetSecurityInfo::AuthDh => auth_flavor(AuthFlavor::AuthDh)(out),
    }
}

#[inline(always)]
fn get_security_info_result<'a, 'b: 'a, W: Write + 'a>(
    value: &'a GetSecurityInfoResult<'b>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        GetSecurityInfoResult::Ok(value) => {
            tuple((error(None), get_security_info_result_ok(value)))(out)
        }
    }
}

#[inline(always)]
fn get_security_info_result_ok<'a, 'b: 'a, W: Write + 'a>(
    value: &'a GetSecurityInfoResultOk<'b>
) -> impl SerializeFn<W> + 'a {
    variable_length_array(&value.0, get_security_info)
}

// Operation 42: EXCHANGE_ID

#[inline(always)]
fn exchange_id_flags<W: Write>(flags: ExchangeIdFlags) -> impl SerializeFn<W> {
    be_u32(flags.bits() as u32)
}

#[inline(always)]
fn exchange_id_result<'a, 'b: 'a, W: Write + 'a>(
    value: &'a Result<ExchangeIdResult<'b>, Error>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(ref value) => tuple((error(None), exchange_id_result_ok(value)))(out),
        Err(value) => error(Some(*value))(out),
    }
}

#[inline(always)]
fn exchange_id_result_ok<'a, 'b: 'a, W: Write + 'a>(
    value: &'a ExchangeIdResult<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((
        be_u64(value.client_id),
        be_u32(value.sequence_id),
        exchange_id_flags(value.flags),
        state_protect_result(&value.state_protect),
        server_owner(&value.server_owner),
        variable_length_opaque(&value.server_scope),
        variable_length_array(&value.server_impl_id, nfs_impl_id),
    ))
}

// Operation 43: CREATE_SESSION

#[inline(always)]
fn channel_attributes<W: Write>(value: ChannelAttributes) -> impl SerializeFn<W> {
    tuple((
        be_u32(value.header_pad_size),
        be_u32(value.max_request_size),
        be_u32(value.max_response_size),
        be_u32(value.max_response_size_cached),
        be_u32(value.max_operations),
        be_u32(value.max_requests),
        variable_length_array(value.rdma_ird.into_iter(), be_u32),
    ))
}

#[inline(always)]
fn create_session_flags<W: Write>(flags: CreateSessionFlags) -> impl SerializeFn<W> {
    be_u32(flags.bits() as u32)
}

#[inline(always)]
fn create_session_result<'a, W: Write + 'a>(
    value: &'a Result<CreateSessionResult, Error>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(ref value) => tuple((error(None), create_session_result_ok(value)))(out),
        Err(value) => error(Some(*value))(out),
    }
}

#[inline(always)]
fn create_session_result_ok<W: Write>(value: &CreateSessionResult) -> impl SerializeFn<W> {
    tuple((
        slice(value.session_id),
        be_u32(value.sequence_id),
        create_session_flags(value.flags),
        channel_attributes(value.fore_channel_attributes),
        channel_attributes(value.back_channel_attributes),
    ))
}

// Operation 44: DESTROY_SESSION

#[inline(always)]
fn destroy_session_result<'a, W: Write + 'a>(
    value: &'a Result<(), Error>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(_) => error(None)(out),
        Err(value) => error(Some(*value))(out),
    }
}

// Operation 52: SECINFO_NO_NAME

#[inline(always)]
fn get_security_info_style<W: Write>(value: GetSecurityInfoStyle) -> impl SerializeFn<W> {
    be_u32(value as u32)
}

#[inline(always)]
fn get_security_info_no_name_args<W: Write>(
    value: GetSecurityInfoNoNameArgs
) -> impl SerializeFn<W> {
    get_security_info_style(value.0)
}

#[inline(always)]
fn get_security_info_no_name_result<'a, 'b: 'a, W: Write + 'a>(
    value: &'a GetSecurityInfoNoNameResult<'b>
) -> impl SerializeFn<W> + 'a {
    get_security_info_result(&value.0)
}

// Operation 53: SEQUENCE

fn sequence_args<W: Write>(value: SequenceArgs) -> impl SerializeFn<W> {
    tuple((
        slice(value.session_id),
        be_u32(value.sequence_id),
        be_u32(value.slot_id),
        be_u32(value.highest_slot_id),
        bool_u32(value.cache_this),
    ))
}

#[inline(always)]
fn sequence_status_flags<W: Write>(flags: SequenceStatusFlags) -> impl SerializeFn<W> {
    be_u32(flags.bits() as u32)
}

#[inline(always)]
fn sequence_result<'a, W: Write + 'a>(
    value: &'a Result<SequenceResult, Error>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(ref value) => tuple((error(None), sequence_result_ok(value)))(out),
        Err(value) => error(Some(*value))(out),
    }
}

#[inline(always)]
fn sequence_result_ok<'a, W: Write + 'a>(value: &'a SequenceResult) -> impl SerializeFn<W> + 'a {
    tuple((
        slice(value.session_id),
        be_u32(value.sequence_id),
        be_u32(value.slot_id),
        be_u32(value.highest_slot_id),
        be_u32(value.target_highest_slot_id),
        sequence_status_flags(value.status_flags),
    ))
}

// Operation 57: DESTROY_CLIENT_ID

#[inline(always)]
fn destroy_client_id_result<'a, W: Write + 'a>(
    value: &'a Result<(), Error>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(_) => error(None)(out),
        Err(value) => error(Some(*value))(out),
    }
}

// Operation 58: RECLAIM_COMPLETE

fn reclaim_complete_args<W: Write>(value: ReclaimCompleteArgs) -> impl SerializeFn<W> {
    bool_u32(value.one_fs)
}

#[inline(always)]
fn reclaim_complete_result<'a, W: Write + 'a>(
    value: &'a Result<(), Error>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Ok(_) => error(None)(out),
        Err(value) => error(Some(*value))(out),
    }
}

//

#[inline(always)]
pub fn message<W: Write>(value: RpcMessage) -> impl SerializeFn<W> {
    tuple((be_u32(value.xid), message_type(value.message_type)))
}

#[inline(always)]
fn message_type<W: Write>(value: MessageType) -> impl SerializeFn<W> {
    be_u32(value as u32)
}

#[inline(always)]
pub fn call<'a, 'b: 'a, W: Write + 'a>(value: &'a Call<'b>) -> impl SerializeFn<W> + 'a {
    move |out| Ok(todo!())
}

#[inline(always)]
pub fn reply<'a, 'b: 'a, W: Write + Seek + 'a>(value: &'a Reply<'b>) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        Reply::Accepted(ref value) => tuple((be_u32(0), accepted_reply(value)))(out),
        Reply::Rejected(ref value) => tuple((be_u32(1), rejected_reply(value)))(out),
    }
}

#[inline(always)]
fn accepted_reply<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'a AcceptedReply<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((opaque_auth(&value.verf), accepted_reply_body(&value.body)))
}

#[inline(always)]
fn accept_status<W: Write>(value: AcceptStatus) -> impl SerializeFn<W> {
    be_u32(value as u32)
}

#[inline(always)]
fn accepted_reply_body<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'a AcceptedReplyBody<'b>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        AcceptedReplyBody::Success(ref value) => {
            tuple((accept_status(AcceptStatus::Success), procedure_reply(value)))(out)
        }
        AcceptedReplyBody::ProgramUnavailable => todo!(),
        AcceptedReplyBody::ProgramMismatch { low, high } => todo!(),
        AcceptedReplyBody::ProcedureUnavailable => todo!(),
        AcceptedReplyBody::GarbageArgs => todo!(),
        AcceptedReplyBody::SystemError => todo!(),
    }
}

#[inline(always)]
fn procedure_reply<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'a ProcedureReply<'b>
) -> impl SerializeFn<W> + 'a {
    move |out| match value {
        ProcedureReply::Null => Ok(out),
        ProcedureReply::Compound(ref value) => compound_result(value)(out),
    }
}

#[inline(always)]
fn rejected_reply<W: Write>(value: &RejectedReply) -> impl SerializeFn<W> {
    move |out| Ok(todo!())
}

#[inline(always)]
fn opaque_auth<'a, 'b: 'a, W: Write + 'a>(value: &'a OpaqueAuth<'b>) -> impl SerializeFn<W> + 'a {
    tuple((
        auth_flavor(value.flavor),
        variable_length_opaque(&value.body),
    ))
}

#[inline(always)]
fn auth_flavor<W: Write>(value: AuthFlavor) -> impl SerializeFn<W> {
    be_u32(value as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cookie_factory::{bytes::be_u16, gen};
    use std::io::Cursor;

    macro_rules! serialize {
        ($serializer:expr, $buffer:ident) => {{
            let cursor = Cursor::new(&mut $buffer[..]);
            let (_, position) = gen($serializer, cursor).unwrap();
            &$buffer[..position as usize]
        }};
    }

    #[test]
    fn test_opaque() {
        let value = &[0x00, 0x01, 0x02, 0x03];
        let mut buffer = [0u8; 16];
        let result = serialize!(opaque(value), buffer);
        assert_eq!(result, &[0x00, 0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_opaque_alignment() {
        let value = &[0x00, 0x01, 0x02, 0x03, 0x04];
        let mut buffer = [0u8; 16];
        let result = serialize!(opaque(value), buffer);
        assert_eq!(result, &[0x00, 0x01, 0x02, 0x03, 0x04, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_variable_length_opaque() {
        let value = &[0x00, 0x01, 0x02, 0x03];
        let mut buffer = [0u8; 16];
        let result = serialize!(variable_length_opaque(value), buffer);
        assert_eq!(result, &[0x00, 0x00, 0x00, 0x04, 0x00, 0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_variable_length_opaque_alignment() {
        let value = &[0x00, 0x01, 0x02, 0x03, 0x04];
        let mut buffer = [0u8; 16];
        let result = serialize!(variable_length_opaque(value), buffer);
        assert_eq!(
            result,
            &[0x00, 0x00, 0x00, 0x05, 0x00, 0x01, 0x02, 0x03, 0x04, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn test_variable_length_array() {
        let value = &[0u16, 1u16, 2u16, 3u16];
        let mut buffer = [0u8; 64];
        let result = serialize!(variable_length_array(value, |i| be_u16(*i)), buffer);
        assert_eq!(result, &[0, 0, 0, 4, 0, 0, 0, 1, 0, 2, 0, 3],);
    }

    #[test]
    fn test_bitmap() {
        let value = Bitmap::from(vec![0x00010203]);
        let mut buffer = [0u8; 16];
        let result = serialize!(bitmap(&value), buffer);
        assert_eq!(result, &[0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_nfs_opnum() {
        let value = NfsOpnum::Access;
        let mut buffer = [0u8; 16];
        let result = serialize!(nfs_opnum(value), buffer);
        assert_eq!(result, &[0x00, 0x00, 0x00, 0x03]);
    }

    #[test]
    fn test_error() {
        let value = Some(Error::BADNAME);
        let mut buffer = [0u8; 16];
        let result = serialize!(error(value), buffer);
        assert_eq!(result, &[0x00, 0x00, 0x27, 0x39]);
    }

    #[test]
    fn test_string() {
        let value = "hello world";
        let mut buffer = [0u8; 16];
        let result = serialize!(string(value), buffer);
        assert_eq!(
            result,
            &[0, 0, 0, 11, b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd', 0x00]
        );
    }

    #[test]
    fn test_time() {
        let value = Time {
            seconds: 123,
            nanoseconds: 456789,
        };
        let mut buffer = [0u8; 16];
        let result = serialize!(time(value), buffer);
        assert_eq!(result, &[0, 0, 0, 0, 0, 0, 0, 123, 0, 6, 248, 85],);
    }

    #[test]
    fn test_nfs_impl_id() {
        let value = NfsImplId {
            domain: "hello".into(),
            name: "world".into(),
            date: Time {
                seconds: 123,
                nanoseconds: 456789,
            },
        };
        let mut buffer = [0u8; 64];
        let result = serialize!(nfs_impl_id(&value), buffer);
        assert_eq!(
            result,
            &[
                0, 0, 0, 5, 104, 101, 108, 108, 111, 0, 0, 0, 0, 0, 0, 5, 119, 111, 114, 108, 100,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 123, 0, 6, 248, 85
            ]
        );
    }

    #[test]
    pub fn test_gss_handle() {
        let value = GssHandle((&[1, 2, 3, 4]).into());
        let mut buffer = [0u8; 8];
        let result = serialize!(gss_handle(&value), buffer);
        assert_eq!(result, &[0, 0, 0, 4, 1, 2, 3, 4]);
    }

    #[test]
    pub fn test_sec_oid() {
        let value = SecOid((&[1, 2, 3, 4]).into());
        let mut buffer = [0u8; 8];
        let result = serialize!(sec_oid(&value), buffer);
        assert_eq!(result, &[0, 0, 0, 4, 1, 2, 3, 4]);
    }

    #[test]
    pub fn test_server_owner() {
        let value = ServerOwner {
            minor_id: 2,
            major_id: (&[1, 2, 3, 4]).into(),
        };
        let mut buffer = [0u8; 16];
        let result = serialize!(server_owner(&value), buffer);
        assert_eq!(result, &[0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 4, 1, 2, 3, 4]);
    }

    #[test]
    pub fn test_client_owner() {
        let value = ClientOwner {
            verifier: [1, 2, 3, 4, 5, 6, 7, 8],
            owner_id: (&[1, 2, 3, 4]).into(),
        };
        let mut buffer = [0u8; 16];
        let result = serialize!(client_owner(&value), buffer);
        assert_eq!(result, &[1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 4, 1, 2, 3, 4]);
    }

    #[test]
    pub fn test_state_protect_ops() {
        let value = StateProtectOps {
            must_enforce: Bitmap::from(vec![1, 2, 3, 4]),
            must_allow: Bitmap::from(vec![5, 6, 7, 8]),
        };
        let mut buffer = [0u8; 64];
        let result = serialize!(state_protect_ops(&value), buffer);
        assert_eq!(
            result,
            &[
                0, 0, 0, 4, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 5,
                0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8
            ]
        );
    }

    #[test]
    pub fn test_file_attributes() {
        let value = vec![AttributeValue::Size(1337), AttributeValue::Change(123456)];
        let mut buffer = [0u8; 64];
        let result = serialize!(file_attributes(&value), buffer);
        assert_eq!(
            result,
            &[
                0x00, 0x00, 0x00, 0x01, 0b00000000, 0b00000000, 0b00000000, 0b00011000, 0x00, 0x00,
                0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x39, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x01, 0xE2, 0x40
            ],
        );
    }
}

use super::*;

use cookie_factory::{
    bytes::{be_i64, be_u32, be_u64},
    combinator::{slice, string},
    multi::many_ref,
    sequence::tuple,
    Seek, SerializeFn, WriteContext,
};
use std::io::Write;

#[inline(always)]
fn utf8str_cis<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: Utf8StrCis<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((be_u32(value.0.len() as u32), string(value.0)))
}

#[inline(always)]
fn utf8str_cs<'a, 'b: 'a, W: Write + Seek + 'a>(value: Utf8StrCs<'b>) -> impl SerializeFn<W> + 'a {
    tuple((be_u32(value.0.len() as u32), string(value.0)))
}

#[inline(always)]
fn opaque<'a, 'b: 'a, W: Write + Seek + 'a>(value: Opaque<'b>) -> impl SerializeFn<W> + 'a {
    slice(value.0)
}

#[inline(always)]
fn variable_length_opaque<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: Opaque<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((be_u32(value.0.len() as u32), slice(value.0)))
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
fn bitmap<'a, 'b: 'a, W: Write + Seek + 'a>(value: &'b Bitmap) -> impl SerializeFn<W> + 'a {
    variable_length_array(&*value.0, |x| be_u32(*x))
}

#[inline(always)]
fn nfs_opnum<W: Write>(value: NfsOpnum) -> impl SerializeFn<W> {
    be_u32(value as u32)
}

#[inline(always)]
fn nfsstat<W: Write>(value: NfsStat) -> impl SerializeFn<W> {
    be_u32(value as u32)
}

#[inline(always)]
fn nfstime<W: Write>(value: NfsTime) -> impl SerializeFn<W> {
    tuple((be_i64(value.seconds), be_u64(value.nseconds)))
}

#[inline(always)]
fn nfs_impl_id<'a, 'b: 'a, W: Write + Seek + 'a>(value: NfsImplId<'b>) -> impl SerializeFn<W> + 'a {
    tuple((
        utf8str_cis(value.domain),
        utf8str_cs(value.name),
        nfstime(value.date),
    ))
}

#[inline(always)]
fn gss_handle<'a, 'b: 'a, W: Write + Seek + 'a>(value: GssHandle<'b>) -> impl SerializeFn<W> + 'a {
    opaque(value.0)
}

#[inline(always)]
fn verifier<'a, 'b: 'a, W: Write + Seek + 'a>(value: Verifier<'b>) -> impl SerializeFn<W> + 'a {
    opaque(value.0)
}

#[inline(always)]
fn sec_oid<'a, 'b: 'a, W: Write + Seek + 'a>(value: SecOid<'b>) -> impl SerializeFn<W> + 'a {
    opaque(value.0)
}

#[inline(always)]
fn client_id<W: Write>(value: ClientId) -> impl SerializeFn<W> {
    be_u64(value.0)
}

#[inline(always)]
fn sequence_id<W: Write>(value: SequenceId) -> impl SerializeFn<W> {
    be_u32(value.0)
}

#[inline(always)]
fn server_owner<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: ServerOwner<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((be_u64(value.minor_id), opaque(value.major_id)))
}

#[inline(always)]
fn client_owner<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: ClientOwner<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((verifier(value.co_verifier), opaque(value.co_ownerid)))
}

#[inline(always)]
fn state_protect_ops<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'b StateProtectOps
) -> impl SerializeFn<W> + 'a {
    tuple((
        bitmap(&value.spo_must_enforce),
        bitmap(&value.spo_must_allow),
    ))
}

#[inline(always)]
fn ssv_sp_parms<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'b SsvSpParms
) -> impl SerializeFn<W> + 'a {
    tuple((
        state_protect_ops(&value.ssp_ops),
        variable_length_array(&value.ssp_hash_algs, |v| sec_oid(v.clone())),
        variable_length_array(&value.ssp_encr_algs, |v| sec_oid(v.clone())),
        be_u32(value.ssp_window),
        be_u32(value.ssp_num_gss_handles),
    ))
}

#[inline(always)]
fn state_protect4_a<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: StateProtectArgs<'b>
) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| match value {
        StateProtectArgs::SP4_NONE => Ok(out),
        StateProtectArgs::SP4_MACH_CRED { ref spa_mach_ops } => {
            state_protect_ops(spa_mach_ops)(out)
        }
        StateProtectArgs::SP4_SSV { ref spa_ssv_parms } => ssv_sp_parms(spa_ssv_parms)(out),
    }
}

#[inline(always)]
fn state_protect4_r<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'b StateProtectResult<'b>
) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| match value {
        StateProtectResult::SP4_NONE => Ok(out),
        StateProtectResult::SP4_MACH_CRED { spa_mach_ops } => state_protect_ops(spa_mach_ops)(out),
        StateProtectResult::SP4_SSV { spa_ssv_info } => ssv_prot_info(spa_ssv_info)(out),
    }
}

#[inline(always)]
fn ssv_prot_info<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'b SsvProtInfo<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((
        state_protect_ops(&value.spi_ops),
        be_u32(value.spi_hash_alg),
        be_u32(value.spi_encr_alg),
        be_u32(value.spi_ssv_len),
        be_u32(value.spi_window),
        variable_length_array(&value.spi_handles, |v| gss_handle(v.clone())),
    ))
}

#[inline(always)]
fn nfs_resop<'a, 'b: 'a, W: Write + Seek + 'a>(value: NfsResOp<'b>) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| match value {
        NfsResOp::OP_EXCHANGE_ID(ref value) => exchange_id_result(value)(out),
    }
}

#[inline(always)]
fn compound_result<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: CompoundResult<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((
        nfsstat(value.status),
        utf8str_cs(value.tag),
        variable_length_array(value.resarray, nfs_resop),
    ))
}

#[inline(always)]
fn exchange_id_result_ok<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'b ExchangeIdResultOk<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((
        client_id(value.clientid),
        sequence_id(value.sequenceid),
        be_u32(value.flags),
        state_protect4_r(&value.state_protect),
        server_owner(value.server_owner.clone()),
        opaque(value.server_scope.clone()),
        variable_length_array(value.server_impl_id.clone(), nfs_impl_id),
    ))
}

#[inline(always)]
fn exchange_id_result<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: &'b ExchangeIdResult<'b>
) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| match value {
        ExchangeIdResult::NFS4_OK(value) => {
            tuple((nfsstat(NfsStat::NFS4_OK), exchange_id_result_ok(value)))(out)
        }
    }
}

#[inline(always)]
pub fn rpc_msg<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: RpcMessage<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((be_u32(value.xid), msg(value.message)))
}

#[inline(always)]
fn msg<'a, 'b: 'a, W: Write + Seek + 'a>(value: Message<'b>) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| match value.clone() {
        Message::Call(value) => tuple((be_u32(0), call(value)))(out),
        Message::Reply(value) => tuple((be_u32(1), reply(value)))(out),
    }
}

#[inline(always)]
fn call<'a, 'b: 'a, W: Write + Seek + 'a>(value: Call<'b>) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| Ok(todo!())
}

#[inline(always)]
fn reply<'a, 'b: 'a, W: Write + Seek + 'a>(value: Reply<'b>) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| match value.clone() {
        Reply::Accepted(value) => tuple((be_u32(0), accepted_reply(value)))(out),
        Reply::Rejected(value) => tuple((be_u32(1), rejected_reply(value)))(out),
    }
}

#[inline(always)]
fn accepted_reply<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: AcceptedReply<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((opaque_auth(value.verf), accepted_reply_body(value.body)))
}

#[inline(always)]
fn accepted_reply_body<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: AcceptedReplyBody<'b>
) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| match value.clone() {
        AcceptedReplyBody::Success(value) => procedure_reply(value)(out),
        AcceptedReplyBody::ProgramUnavailable => todo!(),
        AcceptedReplyBody::ProgramMismatch { low, high } => todo!(),
        AcceptedReplyBody::ProcedureUnavailable => todo!(),
        AcceptedReplyBody::GarbageArgs => todo!(),
        AcceptedReplyBody::SystemError => todo!(),
    }
}

#[inline(always)]
fn procedure_reply<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: ProcedureReply<'b>
) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| match value.clone() {
        ProcedureReply::Null => be_u32(0)(out),
        ProcedureReply::Compound(value) => tuple((be_u32(1), compound_result(value)))(out),
    }
}

#[inline(always)]
fn rejected_reply<W: Write>(value: RejectedReply) -> impl SerializeFn<W> {
    move |out: WriteContext<W>| Ok(todo!())
}

#[inline(always)]
fn opaque_auth<'a, 'b: 'a, W: Write + Seek + 'a>(
    value: OpaqueAuth<'b>
) -> impl SerializeFn<W> + 'a {
    tuple((
        auth_flavor(value.flavor),
        variable_length_opaque(value.body),
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
        let value = Opaque::from(&[0x00, 0x01, 0x02, 0x03]);
        let mut buffer = [0u8; 16];
        let result = serialize!(opaque(value), buffer);
        assert_eq!(result, &[0x00, 0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_variable_length_opaque() {
        let value = Opaque::from(&[0x00, 0x01, 0x02, 0x03]);
        let mut buffer = [0u8; 16];
        let result = serialize!(variable_length_opaque(value), buffer);
        assert_eq!(result, &[0x00, 0x00, 0x00, 0x04, 0x00, 0x01, 0x02, 0x03]);
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
        let value = NfsOpnum::OP_ACCESS;
        let mut buffer = [0u8; 16];
        let result = serialize!(nfs_opnum(value), buffer);
        assert_eq!(result, &[0x00, 0x00, 0x00, 0x03]);
    }

    #[test]
    fn test_nfsstat() {
        let value = NfsStat::NFS4ERR_BADNAME;
        let mut buffer = [0u8; 16];
        let result = serialize!(nfsstat(value), buffer);
        assert_eq!(result, &[0x00, 0x00, 0x27, 0x39]);
    }

    #[test]
    fn test_utf8str_cis() {
        let value = Utf8StrCis::from("hello world");
        let mut buffer = [0u8; 16];
        let result = serialize!(utf8str_cis(value), buffer);
        assert_eq!(
            result,
            &[0, 0, 0, 11, b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd']
        );
    }

    #[test]
    fn test_utf8str_cs() {
        let value = Utf8StrCs::from("hello world");
        let mut buffer = [0u8; 16];
        let result = serialize!(utf8str_cs(value), buffer);
        assert_eq!(
            result,
            &[0, 0, 0, 11, b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd']
        );
    }

    #[test]
    fn test_nfstime() {
        let value = NfsTime {
            seconds: 123,
            nseconds: 456789,
        };
        let mut buffer = [0u8; 16];
        let result = serialize!(nfstime(value), buffer);
        assert_eq!(
            result,
            &[0, 0, 0, 0, 0, 0, 0, 123, 0, 0, 0, 0, 0, 6, 248, 85],
        );
    }

    #[test]
    fn test_nfs_impl_id() {
        let value = NfsImplId {
            domain: Utf8StrCis::from("hello"),
            name: Utf8StrCs::from("world"),
            date: NfsTime {
                seconds: 123,
                nseconds: 456789,
            },
        };
        let mut buffer = [0u8; 64];
        let result = serialize!(nfs_impl_id(value), buffer);
        assert_eq!(
            result,
            &[
                0, 0, 0, 5, 104, 101, 108, 108, 111, 0, 0, 0, 5, 119, 111, 114, 108, 100, 0, 0, 0,
                0, 0, 0, 0, 123, 0, 0, 0, 0, 0, 6, 248, 85
            ]
        );
    }

    #[test]
    pub fn test_gss_handle() {
        let value = GssHandle(Opaque::from(&[1, 2, 3, 4]));
        let mut buffer = [0u8; 8];
        let result = serialize!(gss_handle(value), buffer);
        assert_eq!(result, &[1, 2, 3, 4]);
    }

    #[test]
    pub fn test_verifier() {
        let value = Verifier(Opaque::from(&[1, 2, 3, 4]));
        let mut buffer = [0u8; 8];
        let result = serialize!(verifier(value), buffer);
        assert_eq!(result, &[1, 2, 3, 4]);
    }

    #[test]
    pub fn test_sec_oid() {
        let value = SecOid(Opaque::from(&[1, 2, 3, 4]));
        let mut buffer = [0u8; 8];
        let result = serialize!(sec_oid(value), buffer);
        assert_eq!(result, &[1, 2, 3, 4]);
    }

    #[test]
    pub fn test_client_id() {
        let value = ClientId(1234);
        let mut buffer = [0u8; 8];
        let result = serialize!(client_id(value), buffer);
        assert_eq!(result, &[0, 0, 0, 0, 0, 0, 4, 210]);
    }

    #[test]
    pub fn test_sequence_id() {
        let value = SequenceId(1234);
        let mut buffer = [0u8; 8];
        let result = serialize!(sequence_id(value), buffer);
        assert_eq!(result, &[0, 0, 4, 210]);
    }

    #[test]
    pub fn test_server_owner() {
        let value = ServerOwner {
            minor_id: 2,
            major_id: Opaque::from(&[1, 2, 3, 4]),
        };
        let mut buffer = [0u8; 16];
        let result = serialize!(server_owner(value), buffer);
        assert_eq!(result, &[0, 0, 0, 0, 0, 0, 0, 2, 1, 2, 3, 4]);
    }

    #[test]
    pub fn test_client_owner() {
        let value = ClientOwner {
            co_verifier: Verifier(Opaque::from(&[5, 6, 7, 8])),
            co_ownerid: Opaque::from(&[1, 2, 3, 4]),
        };
        let mut buffer = [0u8; 16];
        let result = serialize!(client_owner(value), buffer);
        assert_eq!(result, &[5, 6, 7, 8, 1, 2, 3, 4]);
    }

    #[test]
    pub fn test_state_protect_ops() {
        let value = StateProtectOps {
            spo_must_enforce: Bitmap::from(vec![1, 2, 3, 4]),
            spo_must_allow: Bitmap::from(vec![5, 6, 7, 8]),
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
}

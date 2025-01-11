#![allow(non_camel_case_types)]

mod decode;
mod encode;

pub use decode::rpc_msg as decode;
pub use encode::rpc_msg as encode;

use bitflags::bitflags;
use derive_more::{From, Into};
use num_derive::FromPrimitive;
use std::borrow::Cow;

const NFS4_FHSIZE: u32 = 128;
const NFS4_VERIFIER_SIZE: u32 = 8;
const NFS4_OPAQUE_LIMIT: u32 = 1024;
const NFS4_SESSIONID_SIZE: u32 = 16;
const NFS4_MAXFILELEN: usize = 0xffffffffffffffff;
const NFS4_MAXFILEOFF: usize = 0xfffffffffffffffe;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ExchangeIdFlags: u32 {
        const SUPP_MOVED_REFER     = 0x00000001;
        const SUPP_MOVED_MIGR      = 0x00000002;
        const SUPP_FENCE_OPS       = 0x00000004;
        const BIND_PRINC_STATEID   = 0x00000100;
        const USE_NON_PNFS         = 0x00010000;
        const USE_PNFS_MDS         = 0x00020000;
        const USE_PNFS_DS          = 0x00040000;
        const MASK_PNFS            = 0x00070000;
        const UPD_CONFIRMED_REC_A  = 0x40000000;
        const CONFIRMED_R          = 0x80000000;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RpcMessage<'a> {
    pub xid: u32,
    pub message: Message<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Message<'a> {
    Call(Call<'a>),
    Reply(Reply<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Call<'a> {
    pub cred: OpaqueAuth<'a>,
    pub verf: OpaqueAuth<'a>,
    pub procedure: ProcedureCall<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Reply<'a> {
    Accepted(AcceptedReply<'a>),
    Rejected(RejectedReply),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AcceptedReply<'a> {
    pub verf: OpaqueAuth<'a>,
    pub body: AcceptedReplyBody<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AcceptedReplyBody<'a> {
    Success(ProcedureReply<'a>),
    ProgramUnavailable,
    ProgramMismatch { low: u32, high: u32 },
    ProcedureUnavailable,
    GarbageArgs,
    SystemError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RejectedReply {
    RpcMismatch { low: u32, high: u32 },
    AuthError { stat: AuthStatus },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub(crate) enum AuthFlavor {
    AUTH_NONE = 0,
    AUTH_SYS = 1,
    AUTH_SHORT = 2,
    AUTH_DH = 3,
    RPCSEC_GSS = 6,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProcedureCall<'a> {
    Null,
    Compound(CompoundArgs<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProcedureReply<'a> {
    Null,
    Compound(CompoundResult<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OpaqueAuth<'a> {
    pub flavor: AuthFlavor,
    pub body: Opaque<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum AuthStatus {
    AUTH_OK = 0, /* success                        */
    /*
     * failed at remote end
     */
    AUTH_BADCRED = 1,      /* bad credential (seal broken)   */
    AUTH_REJECTEDCRED = 2, /* client must begin new session  */
    AUTH_BADVERF = 3,      /* bad verifier (seal broken)     */
    AUTH_REJECTEDVERF = 4, /* verifier expired or replayed   */
    AUTH_TOOWEAK = 5,      /* rejected for security reasons  */
    /*
     * failed locally
     */
    AUTH_INVALIDRESP = 6, /* bogus response verifier        */
    AUTH_FAILED = 7,      /* reason unknown                 */
    /*
     * AUTH_KERB errors; deprecated.  See [RFC2695]
     */
    AUTH_KERB_GENERIC = 8, /* kerberos generic error */
    AUTH_TIMEEXPIRE = 9,   /* time of credential expired */
    AUTH_TKT_FILE = 10,    /* problem with ticket file */
    AUTH_DECODE = 11,      /* can't decode authenticator */
    AUTH_NET_ADDR = 12,    /* wrong net address in ticket */
    /*
     * RPCSEC_GSS GSS related errors
     */
    RPCSEC_GSS_CREDPROBLEM = 13, /* no credentials for user */
    RPCSEC_GSS_CTXPROBLEM = 14,  /* problem with context */
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opaque<'a>(Cow<'a, [u8]>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitmap<'a>(Cow<'a, [u32]>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GssHandle<'a>(Opaque<'a>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utf8StrCis<'a>(Cow<'a, str>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utf8StrCs<'a>(Cow<'a, str>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verifier<'a>(Opaque<'a>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecOid<'a>(Opaque<'a>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct ClientId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct SequenceId(u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerOwner<'a> {
    pub minor_id: u64,
    pub major_id: Opaque<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientOwner<'a> {
    pub verifier: Verifier<'a>,
    pub owner_id: Opaque<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsvSpParms<'a> {
    pub ops: StateProtectOps<'a>,
    pub hash_algs: Vec<SecOid<'a>>,
    pub encr_algs: Vec<SecOid<'a>>,
    pub window: u32,
    pub num_gss_handles: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SsvProtInfo<'a> {
    pub ops: StateProtectOps<'a>,
    pub hash_alg: u32,
    pub encr_alg: u32,
    pub ssv_len: u32,
    pub window: u32,
    pub handles: Vec<GssHandle<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateProtectOps<'a> {
    pub must_enforce: Bitmap<'a>,
    pub must_allow: Bitmap<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time {
    pub seconds: i64,
    pub nanoseconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NfsImplId<'a> {
    pub domain: Utf8StrCis<'a>,
    pub name: Utf8StrCs<'a>,
    pub date: Time,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundArgs<'a> {
    pub tag: Utf8StrCs<'a>,
    pub minorversion: u32,
    pub argarray: Vec<NfsArgOp<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundResult<'a> {
    pub status: Status,
    pub tag: Utf8StrCs<'a>,
    pub resarray: Vec<NfsResOp<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeIdArgs<'a> {
    pub clientowner: ClientOwner<'a>,
    pub flags: ExchangeIdFlags,
    pub state_protect: StateProtectArgs<'a>,
    pub client_impl_id: Option<NfsImplId<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExchangeIdResult<'a> {
    NFS4_OK(ExchangeIdResultOk<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeIdResultOk<'a> {
    pub client_id: ClientId,
    pub sequence_id: SequenceId,
    pub flags: ExchangeIdFlags,
    pub state_protect: StateProtectResult<'a>,
    pub server_owner: ServerOwner<'a>,
    pub server_scope: Opaque<'a>, // max NFS4_OPAQUE_LIMIT
    pub server_impl_id: Option<NfsImplId<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
enum AcceptStatus {
    Success = 0,              /* RPC executed successfully       */
    ProgramUnavailable = 1,   /* remote hasn't exported program  */
    ProgramMismatch = 2,      /* remote can't support version #  */
    ProcedureUnavailable = 3, /* program can't support procedure */
    GarbageArgs = 4,          /* procedure can't decode params   */
    SystemError = 5,          /* e.g. memory allocation failure  */
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
enum RejectStatus {
    RpcMismatch = 0, /* RPC version number != 2          */
    AuthError = 1,   /* remote can't authenticate caller */
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum StateProtectHow {
    None = 0,
    MachineCredentials = 1,
    ServerSideValidation = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateProtectArgs<'a> {
    None,
    MachineCredentials(StateProtectOps<'a>),
    ServerSideValidation(SsvSpParms<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateProtectResult<'a> {
    None,
    MachineCredentials(StateProtectOps<'a>),
    ServerSideValidation(SsvProtInfo<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NfsArgOp<'a> {
    //ACCESS(ACCESS4args),
    //CLOSE(CLOSE4args),
    //COMMIT(COMMIT4args),
    //CREATE(CREATE4args),
    //DELEGPURGE(DELEGPURGE4args),
    //DELEGRETURN(DELEGRETURN4args),
    //GETATTR(GETATTR4args),
    GETFH,
    //LINK(LINK4args),
    //LOCK(LOCK4args),
    //LOCKT(LOCKT4args),
    //LOCKU(LOCKU4args),
    //LOOKUP(LOOKUP4args),
    LOOKUPP,
    //NVERIFY(NVERIFY4args),
    //OPEN(OPEN4args),
    //OPENATTR(OPENATTR4args),
    //OPEN_CONFIRM(OPEN_CONFIRM4args),
    //OPEN_DOWNGRADE(OPEN_DOWNGRADE4args),
    //PUTFH(PUTFH4args),
    PUTPUBFH,
    PUTROOTFH,
    //READ(READ4args),
    //READDIR(READDIR4args),
    READLINK,
    //REMOVE(REMOVE4args),
    //RENAME(RENAME4args),
    //RENEW(RENEW4args),
    RESTOREFH,
    SAVEFH,
    //SECINFO(SECINFO4args),
    //SETATTR(SETATTR4args),
    //SETCLIENTID(SETCLIENTID4args),
    //SETCLIENTID_CONFIRM(SETCLIENTID_CONFIRM4args),
    //VERIFY(VERIFY4args),
    //WRITE(WRITE4args),
    //RELEASE_LOCKOWNER(RELEASE_LOCKOWNER4args),
    //BACKCHANNEL_CTL(BACKCHANNEL_CTL4args),
    //BIND_CONN_TO_SESSION(BIND_CONN_TO_SESSION4args),
    ExchangeId(ExchangeIdArgs<'a>),
    CREATE_SESSION,
    //DESTROY_SESSION(DESTROY_SESSION4args),
    //FREE_STATEID(FREE_STATEID4args),
    //GET_DIR_DELEGATION(GET_DIR_DELEGATION4args),
    //GETDEVICEINFO(GETDEVICEINFO4args),
    //GETDEVICELIST(GETDEVICELIST4args),
    //LAYOUTCOMMIT(LAYOUTCOMMIT4args),
    //LAYOUTGET(LAYOUTGET4args),
    //LAYOUTRETURN(LAYOUTRETURN4args),
    //SECINFO_NO_NAME(SECINFO_NO_NAME4args),
    //SEQUENCE(SEQUENCE4args),
    //SET_SSV(SET_SSV4args),
    //TEST_STATEID(TEST_STATEID4args),
    //WANT_DELEGATION(WANT_DELEGATION4args),
    //DESTROY_CLIENTID(DESTROY_CLIENTID4args),
    //RECLAIM_COMPLETE(RECLAIM_COMPLETE4args),
    ILLEGAL,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NfsResOp<'a> {
    //ACCESS(ACCESS4res),
    //CLOSE(CLOSE4res),
    //COMMIT(COMMIT4res),
    //CREATE(CREATE4res),
    //DELEGPURGE(DELEGPURGE4res),
    //DELEGRETURN(DELEGRETURN4res),
    //GETATTR(GETATTR4res),
    //GETFH(GETFH4res),
    //LINK(LINK4res),
    //LOCK(LOCK4res),
    //LOCKT(LOCKT4res),
    //LOCKU(LOCKU4res),
    //LOOKUP(LOOKUP4res),
    //LOOKUPP(LOOKUPP4res),
    //NVERIFY(NVERIFY4res),
    //OPEN(OPEN4res),
    //OPENATTR(OPENATTR4res),
    //OPEN_CONFIRM(OPEN_CONFIRM4res),
    //OPEN_DOWNGRADE(OPEN_DOWNGRADE4res),
    //PUTFH(PUTFH4res),
    //PUTPUBFH(PUTPUBFH4res),
    //PUTROOTFH(PUTROOTFH4res),
    //READ(READ4res),
    //READDIR(READDIR4res),
    //READLINK(READLINK4res),
    //REMOVE(REMOVE4res),
    //RENAME(RENAME4res),
    //RENEW(RENEW4res),
    //RESTOREFH(RESTOREFH4res),
    //SAVEFH(SAVEFH4res),
    //SECINFO(SECINFO4res),
    //SETATTR(SETATTR4res),
    //SETCLIENTID(SETCLIENTID4res),
    //SETCLIENTID_CONFIRM(SETCLIENTID_CONFIRM4res),
    //VERIFY(VERIFY4res),
    //WRITE(WRITE4res),
    //RELEASE_LOCKOWNER(RELEASE_LOCKOWNER4res),
    //BACKCHANNEL_CTL(BACKCHANNEL_CTL4res),
    //BIND_CONN_TO_SESSION(BIND_CONN_TO_SESSION4res),
    ExchangeId(ExchangeIdResult<'a>),
    //CREATE_SESSION(CREATE_SESSION4res),
    //DESTROY_SESSION(DESTROY_SESSION4res),
    //FREE_STATEID(FREE_STATEID4res),
    //GET_DIR_DELEGATION(GET_DIR_DELEGATION4res),
    //GETDEVICEINFO(GETDEVICEINFO4res),
    //GETDEVICELIST(GETDEVICELIST4res),
    //LAYOUTCOMMIT(LAYOUTCOMMIT4res),
    //LAYOUTGET(LAYOUTGET4res),
    //LAYOUTRETURN(LAYOUTRETURN4res),
    //SECINFO_NO_NAME(SECINFO_NO_NAME4res),
    //SEQUENCE(SEQUENCE4res),
    //SET_SSV(SET_SSV4res),
    //TEST_STATEID(TEST_STATEID4res),
    //WANT_DELEGATION(WANT_DELEGATION4res),
    //DESTROY_CLIENTID(DESTROY_CLIENTID4res),
    //RECLAIM_COMPLETE(RECLAIM_COMPLETE4res),
    //ILLEGAL(ILLEGAL4res),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum Status {
    NFS4_OK = 0,         /* everything is okay       */
    NFS4ERR_PERM = 1,    /* caller not privileged    */
    NFS4ERR_NOENT = 2,   /* no such file/directory   */
    NFS4ERR_IO = 5,      /* hard I/O error           */
    NFS4ERR_NXIO = 6,    /* no such device           */
    NFS4ERR_ACCESS = 13, /* access denied            */
    NFS4ERR_EXIST = 17,  /* file already exists      */
    NFS4ERR_XDEV = 18,   /* different file systems   */
    /* Unused/reserved 19                                            */
    NFS4ERR_NOTDIR = 20,                 /* should be a directory    */
    NFS4ERR_ISDIR = 21,                  /* should not be directory  */
    NFS4ERR_INVAL = 22,                  /* invalid argument         */
    NFS4ERR_FBIG = 27,                   /* file exceeds server max  */
    NFS4ERR_NOSPC = 28,                  /* no space on file system  */
    NFS4ERR_ROFS = 30,                   /* read-only file system    */
    NFS4ERR_MLINK = 31,                  /* too many hard links      */
    NFS4ERR_NAMETOOLONG = 63,            /* name exceeds server max  */
    NFS4ERR_NOTEMPTY = 66,               /* directory not empty      */
    NFS4ERR_DQUOT = 69,                  /* hard quota limit reached */
    NFS4ERR_STALE = 70,                  /* file no longer exists    */
    NFS4ERR_BADHANDLE = 10001,           /* Illegal filehandle       */
    NFS4ERR_BAD_COOKIE = 10003,          /* READDIR cookie is stale  */
    NFS4ERR_NOTSUPP = 10004,             /* operation not supported  */
    NFS4ERR_TOOSMALL = 10005,            /* response limit exceeded  */
    NFS4ERR_SERVERFAULT = 10006,         /* undefined server error   */
    NFS4ERR_BADTYPE = 10007,             /* type invalid for CREATE  */
    NFS4ERR_DELAY = 10008,               /* file "busy" - retry      */
    NFS4ERR_SAME = 10009,                /* nverify says attrs same  */
    NFS4ERR_DENIED = 10010,              /* lock unavailable         */
    NFS4ERR_EXPIRED = 10011,             /* lock lease expired       */
    NFS4ERR_LOCKED = 10012,              /* I/O failed due to lock   */
    NFS4ERR_GRACE = 10013,               /* in grace period          */
    NFS4ERR_FHEXPIRED = 10014,           /* filehandle expired       */
    NFS4ERR_SHARE_DENIED = 10015,        /* share reserve denied     */
    NFS4ERR_WRONGSEC = 10016,            /* wrong security flavor    */
    NFS4ERR_CLID_INUSE = 10017,          /* clientid in use          */
    NFS4ERR_RESOURCE = 10018,            /* resource exhaustion      */
    NFS4ERR_MOVED = 10019,               /* file system relocated    */
    NFS4ERR_NOFILEHANDLE = 10020,        /* current FH is not set    */
    NFS4ERR_MINOR_VERS_MISMATCH = 10021, /* minor vers not supp      */
    NFS4ERR_STALE_CLIENTID = 10022,      /* server has rebooted      */
    NFS4ERR_STALE_STATEID = 10023,       /* server has rebooted      */
    NFS4ERR_OLD_STATEID = 10024,         /* state is out of sync     */
    NFS4ERR_BAD_STATEID = 10025,         /* incorrect stateid        */
    NFS4ERR_BAD_SEQID = 10026,           /* request is out of seq.   */
    NFS4ERR_NOT_SAME = 10027,            /* verify - attrs not same  */
    NFS4ERR_LOCK_RANGE = 10028,          /* lock range not supported */
    NFS4ERR_SYMLINK = 10029,             /* should be file/directory */
    NFS4ERR_RESTOREFH = 10030,           /* no saved filehandle      */
    NFS4ERR_LEASE_MOVED = 10031,         /* some file system moved   */
    NFS4ERR_ATTRNOTSUPP = 10032,         /* recommended attr not sup */
    NFS4ERR_NO_GRACE = 10033,            /* reclaim outside of grace */
    NFS4ERR_RECLAIM_BAD = 10034,         /* reclaim error at server  */
    NFS4ERR_RECLAIM_CONFLICT = 10035,    /* conflict on reclaim      */
    NFS4ERR_BADXDR = 10036,              /* XDR decode failed        */
    NFS4ERR_LOCKS_HELD = 10037,          /* file locks held at CLOSE */
    NFS4ERR_OPENMODE = 10038,            /* conflict in OPEN and I/O */
    NFS4ERR_BADOWNER = 10039,            /* owner translation bad    */
    NFS4ERR_BADCHAR = 10040,             /* UTF-8 char not supported */
    NFS4ERR_BADNAME = 10041,             /* name not supported       */
    NFS4ERR_BAD_RANGE = 10042,           /* lock range not supported */
    NFS4ERR_LOCK_NOTSUPP = 10043,        /* no atomic up/downgrade   */
    NFS4ERR_OP_ILLEGAL = 10044,          /* undefined operation      */
    NFS4ERR_DEADLOCK = 10045,            /* file locking deadlock    */
    NFS4ERR_FILE_OPEN = 10046,           /* open file blocks op.     */
    NFS4ERR_ADMIN_REVOKED = 10047,       /* lock-owner state revoked */
    NFS4ERR_CB_PATH_DOWN = 10048,        /* callback path down       */
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum NfsOpnum {
    ACCESS = 3,
    CLOSE = 4,
    COMMIT = 5,
    CREATE = 6,
    DELEGPURGE = 7,
    DELEGRETURN = 8,
    GETATTR = 9,
    GETFH = 10,
    LINK = 11,
    LOCK = 12,
    LOCKT = 13,
    LOCKU = 14,
    LOOKUP = 15,
    LOOKUPP = 16,
    NVERIFY = 17,
    OPEN = 18,
    OPENATTR = 19,
    OPEN_CONFIRM = 20,
    OPEN_DOWNGRADE = 21,
    PUTFH = 22,
    PUTPUBFH = 23,
    PUTROOTFH = 24,
    READ = 25,
    READDIR = 26,
    READLINK = 27,
    REMOVE = 28,
    RENAME = 29,
    RENEW = 30, /* Mandatory not-to-implement */
    RESTOREFH = 31,
    SAVEFH = 32,
    SECINFO = 33,
    SETATTR = 34,
    SETCLIENTID = 35,         /* Mandatory not-to-implement */
    SETCLIENTID_CONFIRM = 36, /* Mandatory not-to-implement */
    VERIFY = 37,
    WRITE = 38,
    RELEASE_LOCKOWNER = 39, /* Mandatory not-to-implement */
    BACKCHANNEL_CTL = 40,
    BIND_CONN_TO_SESSION = 41,
    ExchangeId = 42,
    CREATE_SESSION = 43,
    DESTROY_SESSION = 44,
    FREE_STATEID = 45,
    GET_DIR_DELEGATION = 46,
    GETDEVICEINFO = 47,
    GETDEVICELIST = 48,
    LAYOUTCOMMIT = 49,
    LAYOUTGET = 50,
    LAYOUTRETURN = 51,
    SECINFO_NO_NAME = 52,
    SEQUENCE = 53,
    SET_SSV = 54,
    TEST_STATEID = 55,
    WANT_DELEGATION = 56,
    DESTROY_CLIENTID = 57,
    RECLAIM_COMPLETE = 58,
    ILLEGAL = 10044,
}

impl<'a, T> From<T> for Opaque<'a>
where
    Cow<'a, [u8]>: From<T>,
{
    fn from(value: T) -> Self {
        Self(Cow::from(value))
    }
}

impl<'a, T> From<T> for Utf8StrCis<'a>
where
    Cow<'a, str>: From<T>,
{
    fn from(value: T) -> Self {
        Self(Cow::from(value))
    }
}

impl<'a, T> From<T> for Utf8StrCs<'a>
where
    Cow<'a, str>: From<T>,
{
    fn from(value: T) -> Self {
        Self(Cow::from(value))
    }
}

impl<'a, T> From<T> for Bitmap<'a>
where
    Cow<'a, [u32]>: From<T>,
{
    fn from(value: T) -> Self {
        Self(Cow::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cookie_factory::gen;
    use std::io::Cursor;

    macro_rules! serialize {
        ($serializer:expr, $buffer:ident) => {{
            let cursor = Cursor::new(&mut $buffer[..]);
            let (_, position) = gen($serializer, cursor).unwrap();
            &$buffer[..position as usize]
        }};
    }

    #[test]
    fn test_encode_decode_rpc_msg() {
        let rpc_msg = RpcMessage {
            xid: 1234,
            message: Message::Reply(Reply::Accepted(AcceptedReply {
                verf: OpaqueAuth {
                    flavor: AuthFlavor::AUTH_NONE, // TODO
                    body: (&[]).into(),            // TODO
                },
                body: AcceptedReplyBody::Success(ProcedureReply::Compound(CompoundResult {
                    status: Status::NFS4_OK,
                    tag: "hello world".into(),
                    resarray: vec![NfsResOp::ExchangeId(ExchangeIdResult::NFS4_OK(
                        ExchangeIdResultOk {
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
                                domain: "domain".into(),
                                name: "name".into(),
                                date: Time {
                                    seconds: 0,
                                    nanoseconds: 0,
                                },
                            }),
                        },
                    ))],
                })),
            })),
        };
        let mut buffer = [0u8; 1024];
        let buffer = serialize!(encode(rpc_msg), buffer);
        let (buffer, decoded) = decode(buffer).unwrap();
    }
}

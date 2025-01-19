#![allow(non_camel_case_types)]

pub mod decode;
pub mod encode;

use bitflags::bitflags;
use derive_more::{Deref, From, Into};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::borrow::Cow;

// RFC 1831

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct AuthSysParms<'a> {
    stamp: u32,
    machine_name: Cow<'a, str>,
    uid: u32,
    gid: u32,
    gids: Vec<u32>,
}

//

const NFS4_FHSIZE: u32 = 128;
const NFS4_VERIFIER_SIZE: u32 = 8;
const NFS4_OPAQUE_LIMIT: u32 = 1024;
const NFS4_SESSIONID_SIZE: u32 = 16;
const NFS4_MAXFILELEN: usize = 0xffffffffffffffff;
const NFS4_MAXFILEOFF: usize = 0xfffffffffffffffe;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum MessageType {
    Call = 0,
    Reply = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RpcMessage {
    pub xid: u32,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call<'a> {
    pub cred: OpaqueAuth<'a>,
    pub verf: OpaqueAuth<'a>,
    pub procedure: ProcedureCall<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply<'a> {
    Accepted(AcceptedReply<'a>),
    Rejected(RejectedReply),
}

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct AcceptedReply<'a> {
    pub verf: OpaqueAuth<'a>,
    pub body: AcceptedReplyBody<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptedReplyBody<'a> {
    Success(ProcedureReply<'a>),
    ProgramUnavailable,
    ProgramMismatch { low: u32, high: u32 },
    ProcedureUnavailable,
    GarbageArgs,
    SystemError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectedReply {
    RpcMismatch { low: u32, high: u32 },
    AuthError { stat: AuthStatus },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum AuthFlavor {
    AuthNone = 0,
    AuthSys = 1,
    AuthShort = 2,
    AuthDh = 3,
    RpcSecGss = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum ProcedureNumber {
    Null = 0,
    Compound = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcedureCall<'a> {
    Null,
    Compound(CompoundArgs<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcedureReply<'a> {
    Null,
    Compound(CompoundResult<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct OpaqueAuth<'a> {
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

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct AclSupportFlags: u32 {
        const ALLOW = 0x00000001;
        const DENY = 0x00000002;
        const AUDIT = 0x00000004;
        const ALARM = 0x00000008;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum Attribute {
    SupportedAttributes = 0,
    Type = 1,
    FileHandleExpireType = 2,
    Change = 3,
    Size = 4,
    LinkSupport = 5,
    SymlinkSupport = 6,
    NamedAttributes = 7,
    FileSystemId = 8,
    UniqueHandles = 9,
    LeaseTime = 10,
    ReadDirAttributeError = 11,
    AclSupport = 13,
    CaseInsensitive = 16,
    CasePreserving = 17,
    FileHandle = 19,
    FileId = 20,
    MaxFileSize = 27,
    MaxRead = 30,
    MaxWrite = 31,
    Mode = 33,
    NumberOfLinks = 35,
    //Owner = 36,
    //OwnerGroup = 37,
    //RawDevice = 41,
    //SpaceUsed = 45,
    //TimeAccess = 47,
    //TimeMetadata = 52,
    //TimeModify = 53,
    MountedOnFileId = 55,
    SupportedAttributesExclusiveCreate = 75,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeValue<'a> {
    SupportedAttributes(Cow<'a, [Attribute]>),
    Type(FileType),
    FileHandleExpireType(u32),
    Change(u64),
    Size(u64),
    LinkSupport(bool),
    SymlinkSupport(bool),
    NamedAttributes(bool),
    FileSystemId(FileSystemId),
    UniqueHandles(bool),
    LeaseTime(u32),
    ReadDirAttributeError, // enum
    AclSupport(AclSupportFlags),
    CaseInsensitive(bool),
    CasePreserving(bool),
    FileHandle(FileHandle<'a>),
    FileId(u64),
    MaxFileSize(u64),
    MaxRead(u64),
    MaxWrite(u64),
    Mode(Mode),
    NumberOfLinks(u32),
    MountedOnFileId(u64),
    SupportedAttributesExclusiveCreate(Cow<'a, [Attribute]>),
}

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct Opaque<'a>(Cow<'a, [u8]>);

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct Bitmap<'a>(Cow<'a, [u32]>);

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct Utf8StrCis<'a>(Cow<'a, str>);

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct Utf8StrCs<'a>(Cow<'a, str>);

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct Component<'a>(Utf8StrCs<'a>);

#[derive(Debug, Clone, PartialEq, Eq, Deref, From, Into)]
pub struct Verifier([u8; 8]);

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct SecOid<'a>(Opaque<'a>);

#[derive(Debug, Clone, PartialEq, Eq, Deref, From, Into)]
pub struct FileHandle<'a>(Opaque<'a>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct ClientId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct SequenceId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct SessionId([u8; 16]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct SlotId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct Mode(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct Qop(u32);

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct AttributeMask<'a>(Cow<'a, [u32]>);

#[derive(Clone)]
pub struct AttributeMaskIntoIter<'a> {
    mask: AttributeMask<'a>,
    cursor: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct ServerOwner<'a> {
    pub minor_id: u64,
    pub major_id: Opaque<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct ClientOwner<'a> {
    pub verifier: Verifier,
    pub owner_id: Opaque<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, From)]
#[from(forward)]
pub struct Time {
    pub seconds: i64,
    pub nanoseconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileAttributes<'a> {
    pub values: Vec<AttributeValue<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NfsImplId<'a> {
    pub domain: Utf8StrCis<'a>,
    pub name: Utf8StrCs<'a>,
    pub date: Time,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSystemId {
    pub major: u64,
    pub minor: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum FileType {
    Regular = 1,            /* Regular File */
    Directory = 2,          /* Directory */
    BlockDevice = 3,        /* Special File - block device */
    CharacterDevice = 4,    /* Special File - character device */
    Symlink = 5,            /* Symbolic Link */
    Socket = 6,             /* Special File - socket */
    Fifo = 7,               /* Special File - fifo */
    AttributeDirectory = 8, /* Attribute Directory */
    NamedAttribute = 9,     /* Named Attribute */
}

// Procedure 1

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum NfsOpnum {
    Access = 3,
    CLOSE = 4,
    COMMIT = 5,
    CREATE = 6,
    DELEGPURGE = 7,
    DELEGRETURN = 8,
    GetAttributes = 9,
    GetFileHandle = 10,
    LINK = 11,
    LOCK = 12,
    LOCKT = 13,
    LOCKU = 14,
    Lookup = 15,
    LOOKUPP = 16,
    NVERIFY = 17,
    OPEN = 18,
    OPENATTR = 19,
    OPEN_CONFIRM = 20,
    OPEN_DOWNGRADE = 21,
    PutFileHandle = 22,
    PUTPUBFH = 23,
    PutRootFileHandle = 24,
    READ = 25,
    ReadDirectory = 26,
    READLINK = 27,
    REMOVE = 28,
    RENAME = 29,
    RENEW = 30, /* Mandatory not-to-implement */
    RESTOREFH = 31,
    SAVEFH = 32,
    GetSecurityInfo = 33,
    SETATTR = 34,
    SETCLIENTID = 35,         /* Mandatory not-to-implement */
    SETCLIENTID_CONFIRM = 36, /* Mandatory not-to-implement */
    VERIFY = 37,
    WRITE = 38,
    RELEASE_LOCKOWNER = 39, /* Mandatory not-to-implement */
    BACKCHANNEL_CTL = 40,
    BIND_CONN_TO_SESSION = 41,
    ExchangeId = 42,
    CreateSession = 43,
    DestroySession = 44,
    FREE_STATEID = 45,
    GET_DIR_DELEGATION = 46,
    GETDEVICEINFO = 47,
    GETDEVICELIST = 48,
    LAYOUTCOMMIT = 49,
    LAYOUTGET = 50,
    LAYOUTRETURN = 51,
    GetSecurityInfoNoName = 52,
    Sequence = 53,
    SET_SSV = 54,
    TEST_STATEID = 55,
    WANT_DELEGATION = 56,
    DestroyClientId = 57,
    ReclaimComplete = 58,
    ILLEGAL = 10044,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NfsArgOp<'a> {
    Access(AccessFlags),
    //CLOSE(CLOSE4args),
    //COMMIT(COMMIT4args),
    //CREATE(CREATE4args),
    //DELEGPURGE(DELEGPURGE4args),
    //DELEGRETURN(DELEGRETURN4args),
    GetAttributes(GetAttributesArgs<'a>),
    GetFileHandle,
    //LINK(LINK4args),
    //LOCK(LOCK4args),
    //LOCKT(LOCKT4args),
    //LOCKU(LOCKU4args),
    Lookup(Component<'a>),
    //LOOKUPP,
    //NVERIFY(NVERIFY4args),
    //OPEN(OPEN4args),
    //OPENATTR(OPENATTR4args),
    //OPEN_CONFIRM(OPEN_CONFIRM4args),
    //OPEN_DOWNGRADE(OPEN_DOWNGRADE4args),
    PutFileHandle(FileHandle<'a>),
    //PUTPUBFH,
    PutRootFileHandle,
    //READ(READ4args),
    ReadDirectory(ReadDirectoryArgs<'a>),
    //READLINK,
    //REMOVE(REMOVE4args),
    //RENAME(RENAME4args),
    //RENEW(RENEW4args),
    //RESTOREFH,
    //SAVEFH,
    GetSecurityInfo(GetSecurityInfoArgs<'a>),
    //SETATTR(SETATTR4args),
    //SETCLIENTID(SETCLIENTID4args),
    //SETCLIENTID_CONFIRM(SETCLIENTID_CONFIRM4args),
    //VERIFY(VERIFY4args),
    //WRITE(WRITE4args),
    //RELEASE_LOCKOWNER(RELEASE_LOCKOWNER4args),
    //BACKCHANNEL_CTL(BACKCHANNEL_CTL4args),
    //BIND_CONN_TO_SESSION(BIND_CONN_TO_SESSION4args),
    ExchangeId(ExchangeIdArgs<'a>),
    CreateSession(CreateSessionArgs<'a>),
    DestroySession(DestroySessionArgs),
    //FREE_STATEID(FREE_STATEID4args),
    //GET_DIR_DELEGATION(GET_DIR_DELEGATION4args),
    //GETDEVICEINFO(GETDEVICEINFO4args),
    //GETDEVICELIST(GETDEVICELIST4args),
    //LAYOUTCOMMIT(LAYOUTCOMMIT4args),
    //LAYOUTGET(LAYOUTGET4args),
    //LAYOUTRETURN(LAYOUTRETURN4args),
    GetSecurityInfoNoName(GetSecurityInfoNoNameArgs),
    Sequence(SequenceArgs),
    //SET_SSV(SET_SSV4args),
    //TEST_STATEID(TEST_STATEID4args),
    //WANT_DELEGATION(WANT_DELEGATION4args),
    DestroyClientId(DestroyClientIdArgs),
    ReclaimComplete(ReclaimCompleteArgs),
    //ILLEGAL,
}

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct CompoundArgs<'a> {
    pub tag: Utf8StrCs<'a>,
    pub minorversion: u32,
    pub argarray: Vec<NfsArgOp<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NfsResOp<'a> {
    Access(Result<AccessResult, Error>),
    //CLOSE(CLOSE4res),
    //COMMIT(COMMIT4res),
    //CREATE(CREATE4res),
    //DELEGPURGE(DELEGPURGE4res),
    //DELEGRETURN(DELEGRETURN4res),
    GetAttributes(Result<FileAttributes<'a>, Error>),
    GetFileHandle(Result<FileHandle<'a>, Error>),
    //LINK(LINK4res),
    //LOCK(LOCK4res),
    //LOCKT(LOCKT4res),
    //LOCKU(LOCKU4res),
    Lookup(Result<(), Error>),
    //LOOKUPP(LOOKUPP4res),
    //NVERIFY(NVERIFY4res),
    //OPEN(OPEN4res),
    //OPENATTR(OPENATTR4res),
    //OPEN_CONFIRM(OPEN_CONFIRM4res),
    //OPEN_DOWNGRADE(OPEN_DOWNGRADE4res),
    PutFileHandle(Result<(), Error>),
    //PUTPUBFH(PUTPUBFH4res),
    PutRootFileHandle(Result<(), Error>),
    //READ(READ4res),
    ReadDirectory(Result<ReadDirectoryResult<'a>, Error>),
    //READLINK(READLINK4res),
    //REMOVE(REMOVE4res),
    //RENAME(RENAME4res),
    //RENEW(RENEW4res),
    //RESTOREFH(RESTOREFH4res),
    //SAVEFH(SAVEFH4res),
    GetSecurityInfo(GetSecurityInfoResult<'a>),
    //SETATTR(SETATTR4res),
    //SETCLIENTID(SETCLIENTID4res),
    //SETCLIENTID_CONFIRM(SETCLIENTID_CONFIRM4res),
    //VERIFY(VERIFY4res),
    //WRITE(WRITE4res),
    //RELEASE_LOCKOWNER(RELEASE_LOCKOWNER4res),
    //BACKCHANNEL_CTL(BACKCHANNEL_CTL4res),
    //BIND_CONN_TO_SESSION(BIND_CONN_TO_SESSION4res),
    ExchangeId(Result<ExchangeIdResult<'a>, Error>),
    CreateSession(Result<CreateSessionResult, Error>),
    DestroySession(Result<(), Error>),
    //FREE_STATEID(FREE_STATEID4res),
    //GET_DIR_DELEGATION(GET_DIR_DELEGATION4res),
    //GETDEVICEINFO(GETDEVICEINFO4res),
    //GETDEVICELIST(GETDEVICELIST4res),
    //LAYOUTCOMMIT(LAYOUTCOMMIT4res),
    //LAYOUTGET(LAYOUTGET4res),
    //LAYOUTRETURN(LAYOUTRETURN4res),
    GetSecurityInfoNoName(GetSecurityInfoNoNameResult<'a>),
    Sequence(Result<SequenceResult, Error>),
    //SET_SSV(SET_SSV4res),
    //TEST_STATEID(TEST_STATEID4res),
    //WANT_DELEGATION(WANT_DELEGATION4res),
    DestroyClientId(Result<(), Error>),
    ReclaimComplete(Result<(), Error>),
    //ILLEGAL(ILLEGAL4res),
}

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct CompoundResult<'a> {
    pub error: Option<Error>,
    pub tag: Utf8StrCs<'a>,
    pub resarray: Vec<NfsResOp<'a>>,
}

// Operation 3: ACCESS

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct AccessFlags: u32 {
        const READ    = 0x00000001;
        const LOOKUP  = 0x00000002;
        const MODIFY  = 0x00000004;
        const EXTEND  = 0x00000008;
        const DELETE  = 0x00000010;
        const EXECUTE = 0x00000020;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct AccessResult {
    pub supported: AccessFlags,
    pub access: AccessFlags,
}

// Operation 9: GETATTR

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct GetAttributesArgs<'a> {
    pub attr_request: Bitmap<'a>,
}

// Operation 10: GETFH

// Operation 15: LOOKUP

// Operation 22: PUTFH

// Operation 24: PUTROOTFH

// Operation 26: READDIR

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct ReadDirectoryArgs<'a> {
    pub cookie: u64,
    pub cookie_verf: Verifier,
    pub dir_count: u32,
    pub max_count: u32,
    pub attr_request: Bitmap<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'a> {
    pub cookie: u64,
    pub name: Component<'a>,
    pub file_attributes: FileAttributes<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryList<'a> {
    pub entries: Vec<Entry<'a>>,
    pub eof: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadDirectoryResult<'a> {
    pub cookie_verf: Verifier,
    pub directory_list: DirectoryList<'a>,
}

// Operation 33: SECINFO

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct GetSecurityInfoArgs<'a> {
    name: Component<'a>,
}

/* RFC 2203 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum RpcGssSvc {
    None = 1,
    Integrity = 2,
    Privacy = 3,
}

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct RpcSecGssInfo<'a> {
    oid: SecOid<'a>,
    qop: Qop,
    service: RpcGssSvc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetSecurityInfo<'a> {
    AuthNone,
    AuthSys,
    AuthShort,
    AuthDh,
    RpcSecGss(RpcSecGssInfo<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetSecurityInfoResult<'a> {
    Ok(GetSecurityInfoResultOk<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetSecurityInfoResultOk<'a>(pub Vec<GetSecurityInfo<'a>>);

// Operation 40

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GssHandle<'a>(Opaque<'a>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GssCallbackHandles<'a> {
    service: RpcGssSvc,
    handle_from_server: GssHandle<'a>,
    handle_from_client: GssHandle<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallbackSecParms<'a> {
    AuthNone,
    AuthSys(AuthSysParms<'a>), /* RFC 1831 */
    RpcSecGss(GssCallbackHandles<'a>),
}

// Operation 42

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

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct StateProtectOps<'a> {
    pub must_enforce: Bitmap<'a>,
    pub must_allow: Bitmap<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct SsvSpParms<'a> {
    pub ops: StateProtectOps<'a>,
    pub hash_algs: Vec<SecOid<'a>>,
    pub encr_algs: Vec<SecOid<'a>>,
    pub window: u32,
    pub num_gss_handles: u32,
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

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct ExchangeIdArgs<'a> {
    pub clientowner: ClientOwner<'a>,
    pub flags: ExchangeIdFlags,
    pub state_protect: StateProtectArgs<'a>,
    pub client_impl_id: Option<NfsImplId<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateProtectResult<'a> {
    None,
    MachineCredentials(StateProtectOps<'a>),
    ServerSideValidation(SsvProtInfo<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsvProtInfo<'a> {
    pub ops: StateProtectOps<'a>,
    pub hash_alg: u32,
    pub encr_alg: u32,
    pub ssv_len: u32,
    pub window: u32,
    pub handles: Vec<GssHandle<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct ExchangeIdResult<'a> {
    pub client_id: ClientId,
    pub sequence_id: SequenceId,
    pub flags: ExchangeIdFlags,
    pub state_protect: StateProtectResult<'a>,
    pub server_owner: ServerOwner<'a>,
    pub server_scope: Opaque<'a>, // max NFS4_OPAQUE_LIMIT
    pub server_impl_id: Option<NfsImplId<'a>>,
}

// Operation 43

#[derive(Debug, Clone, Copy, PartialEq, Eq, From)]
#[from(forward)]
pub struct ChannelAttributes {
    pub header_pad_size: u32,
    pub max_request_size: u32,
    pub max_response_size: u32,
    pub max_response_size_cached: u32,
    pub max_operations: u32,
    pub max_requests: u32,
    pub rdma_ird: Option<u32>,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CreateSessionFlags: u32 {
        const PERSIST        = 0x00000001;
        const CONN_BACK_CHAN = 0x00000002;
        const CONN_RDMA      = 0x00000004;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct CreateSessionArgs<'a> {
    pub client_id: ClientId,
    pub sequence_id: SequenceId,
    pub flags: CreateSessionFlags,
    pub fore_channel_attributes: ChannelAttributes,
    pub back_channel_attributes: ChannelAttributes,
    pub cb_program: u32,
    pub sec_parms: Vec<CallbackSecParms<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct CreateSessionResult {
    pub session_id: SessionId,
    pub sequence_id: SequenceId,
    pub flags: CreateSessionFlags,
    pub fore_channel_attributes: ChannelAttributes,
    pub back_channel_attributes: ChannelAttributes,
}

// Operation 44

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestroySessionArgs {
    pub session_id: SessionId,
}

// Operation 52: SECINFO_NO_NAME

#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive)]
pub enum GetSecurityInfoStyle {
    CurrentFileHandle = 0,
    Parent = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetSecurityInfoNoNameArgs(pub GetSecurityInfoStyle);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetSecurityInfoNoNameResult<'a>(pub GetSecurityInfoResult<'a>);

// Operation 53

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct SequenceArgs {
    pub session_id: SessionId,
    pub sequence_id: SequenceId,
    pub slot_id: SlotId,
    pub highest_slot_id: SlotId,
    pub cache_this: bool,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SequenceStatusFlags: u32 {
        const CB_PATH_DOWN               = 0x00000001;
        const CB_GSS_CONTEXTS_EXPIRING   = 0x00000002;
        const CB_GSS_CONTEXTS_EXPIRED    = 0x00000004;
        const EXPIRED_ALL_STATE_REVOKED  = 0x00000008;
        const EXPIRED_SOME_STATE_REVOKED = 0x00000010;
        const ADMIN_STATE_REVOKED        = 0x00000020;
        const RECALLABLE_STATE_REVOKED   = 0x00000040;
        const LEASE_MOVED                = 0x00000080;
        const RESTART_RECLAIM_NEEDED     = 0x00000100;
        const CB_PATH_DOWN_SESSION       = 0x00000200;
        const BACKCHANNEL_FAULT          = 0x00000400;
        const DEVID_CHANGED              = 0x00000800;
        const DEVID_DELETED              = 0x00001000;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, From)]
#[from(forward)]
pub struct SequenceResult {
    pub session_id: SessionId,
    pub sequence_id: SequenceId,
    pub slot_id: SlotId,
    pub highest_slot_id: SlotId,
    pub target_highest_slot_id: SlotId,
    pub status_flags: SequenceStatusFlags,
}

// Operation 57

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestroyClientIdArgs {
    pub client_id: ClientId,
}

// Operation 58

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReclaimCompleteArgs {
    pub one_fs: bool,
}

//

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
pub enum Error {
    PERM = 1,                    /* caller not privileged    */
    NOENT = 2,                   /* no such file/directory   */
    IO = 5,                      /* hard I/O error           */
    NXIO = 6,                    /* no such device           */
    Access = 13,                 /* access denied            */
    EXIST = 17,                  /* file already exists      */
    XDEV = 18,                   /* different file systems   */
    NOTDIR = 20,                 /* should be a directory    */
    ISDIR = 21,                  /* should not be directory  */
    INVAL = 22,                  /* invalid argument         */
    FBIG = 27,                   /* file exceeds server max  */
    NOSPC = 28,                  /* no space on file system  */
    ROFS = 30,                   /* read-only file system    */
    MLINK = 31,                  /* too many hard links      */
    NAMETOOLONG = 63,            /* name exceeds server max  */
    NOTEMPTY = 66,               /* directory not empty      */
    DQUOT = 69,                  /* hard quota limit reached */
    STALE = 70,                  /* file no longer exists    */
    BADHANDLE = 10001,           /* Illegal filehandle       */
    BAD_COOKIE = 10003,          /* READDIR cookie is stale  */
    NOTSUPP = 10004,             /* operation not supported  */
    TOOSMALL = 10005,            /* response limit exceeded  */
    SERVERFAULT = 10006,         /* undefined server error   */
    BADTYPE = 10007,             /* type invalid for CREATE  */
    DELAY = 10008,               /* file "busy" - retry      */
    SAME = 10009,                /* nverify says attrs same  */
    DENIED = 10010,              /* lock unavailable         */
    EXPIRED = 10011,             /* lock lease expired       */
    LOCKED = 10012,              /* I/O failed due to lock   */
    GRACE = 10013,               /* in grace period          */
    FHEXPIRED = 10014,           /* filehandle expired       */
    SHARE_DENIED = 10015,        /* share reserve denied     */
    WRONGSEC = 10016,            /* wrong security flavor    */
    CLID_INUSE = 10017,          /* clientid in use          */
    RESOURCE = 10018,            /* resource exhaustion      */
    MOVED = 10019,               /* file system relocated    */
    NOFILEHANDLE = 10020,        /* current FH is not set    */
    MINOR_VERS_MISMATCH = 10021, /* minor vers not supp      */
    STALE_CLIENTID = 10022,      /* server has rebooted      */
    STALE_STATEID = 10023,       /* server has rebooted      */
    OLD_STATEID = 10024,         /* state is out of sync     */
    BAD_STATEID = 10025,         /* incorrect stateid        */
    BAD_SEQID = 10026,           /* request is out of seq.   */
    NOT_SAME = 10027,            /* verify - attrs not same  */
    LOCK_RANGE = 10028,          /* lock range not supported */
    SYMLINK = 10029,             /* should be file/directory */
    RESTOREFH = 10030,           /* no saved filehandle      */
    LEASE_MOVED = 10031,         /* some file system moved   */
    ATTRNOTSUPP = 10032,         /* recommended attr not sup */
    NO_GRACE = 10033,            /* reclaim outside of grace */
    RECLAIM_BAD = 10034,         /* reclaim error at server  */
    RECLAIM_CONFLICT = 10035,    /* conflict on reclaim      */
    BADXDR = 10036,              /* XDR decode failed        */
    LOCKS_HELD = 10037,          /* file locks held at CLOSE */
    OPENMODE = 10038,            /* conflict in OPEN and I/O */
    BADOWNER = 10039,            /* owner translation bad    */
    BADCHAR = 10040,             /* UTF-8 char not supported */
    BADNAME = 10041,             /* name not supported       */
    BAD_RANGE = 10042,           /* lock range not supported */
    LOCK_NOTSUPP = 10043,        /* no atomic up/downgrade   */
    OP_ILLEGAL = 10044,          /* undefined operation      */
    DEADLOCK = 10045,            /* file locking deadlock    */
    FILE_OPEN = 10046,           /* open file blocks op.     */
    ADMIN_REVOKED = 10047,       /* lock-owner state revoked */
    CB_PATH_DOWN = 10048,        /* callback path down       */
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

impl<'a, T> From<T> for AttributeMask<'a>
where
    Cow<'a, [u32]>: From<T>,
{
    fn from(value: T) -> Self {
        Self(Cow::from(value))
    }
}

impl<'a> IntoIterator for AttributeMask<'a> {
    type Item = Attribute;
    type IntoIter = AttributeMaskIntoIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            mask: self,
            cursor: 0,
        }
    }
}

impl Iterator for AttributeMaskIntoIter<'_> {
    type Item = Attribute;

    fn next(&mut self) -> Option<Self::Item> {
        while self.cursor < self.mask.len() * 32 {
            let cursor = self.cursor;
            self.cursor += 1;
            if (self.mask[cursor / 32] & (1 << (cursor % 32))) == 0 {
                continue;
            }
            let attribute @ Some(_) = Attribute::from_usize(cursor) else {
                continue;
            };
            return attribute;
        }
        None
    }
}

impl<'a, T> From<T> for Component<'a>
where
    Utf8StrCs<'a>: From<T>,
{
    fn from(value: T) -> Self {
        Self(Utf8StrCs::from(value))
    }
}

impl AttributeValue<'_> {
    #[inline]
    fn attribute(&self) -> Attribute {
        match self {
            Self::SupportedAttributes(_) => Attribute::SupportedAttributes,
            Self::Type(_) => Attribute::Type,
            Self::FileHandleExpireType(_) => Attribute::FileHandleExpireType,
            Self::Change(_) => Attribute::Change,
            Self::Size(_) => Attribute::Size,
            Self::LinkSupport(_) => Attribute::LinkSupport,
            Self::SymlinkSupport(_) => Attribute::SymlinkSupport,
            Self::NamedAttributes(_) => Attribute::NamedAttributes,
            Self::FileSystemId(_) => Attribute::FileSystemId,
            Self::UniqueHandles(_) => Attribute::UniqueHandles,
            Self::LeaseTime(_) => Attribute::LeaseTime,
            Self::ReadDirAttributeError => Attribute::ReadDirAttributeError,
            Self::AclSupport(_) => Attribute::AclSupport,
            Self::CaseInsensitive(_) => Attribute::CaseInsensitive,
            Self::CasePreserving(_) => Attribute::CasePreserving,
            Self::FileHandle(_) => Attribute::FileHandle,
            Self::FileId(_) => Attribute::FileId,
            Self::MaxFileSize(_) => Attribute::MaxFileSize,
            Self::MaxRead(_) => Attribute::MaxRead,
            Self::MaxWrite(_) => Attribute::MaxWrite,
            Self::Mode(_) => Attribute::Mode,
            Self::NumberOfLinks(_) => Attribute::NumberOfLinks,
            Self::MountedOnFileId(_) => Attribute::MountedOnFileId,
            Self::SupportedAttributesExclusiveCreate(_) => {
                Attribute::SupportedAttributesExclusiveCreate
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cookie_factory::gen;
    use cookie_factory::sequence::tuple;
    use std::io::Cursor;

    macro_rules! serialize {
        ($serializer:expr, $buffer:ident) => {{
            let cursor = Cursor::new(&mut $buffer[..]);
            let (_, position) = gen($serializer, cursor).unwrap();
            &$buffer[..position as usize]
        }};
    }

    #[test]
    fn test_encode_decode_rpc_msg_reply() {
        let message = RpcMessage {
            xid: 1234,
            message_type: MessageType::Reply,
        };
        let reply = Reply::Accepted(AcceptedReply {
            verf: OpaqueAuth {
                flavor: AuthFlavor::AuthNone, // TODO
                body: (&[]).into(),           // TODO
            },
            body: AcceptedReplyBody::Success(ProcedureReply::Compound(CompoundResult {
                error: None,
                tag: "hello world".into(),
                resarray: vec![
                    NfsResOp::GetFileHandle(Ok(FileHandle::from(Opaque::from(&[2; 128])))),
                    NfsResOp::PutRootFileHandle(Err(Error::WRONGSEC)),
                    NfsResOp::ExchangeId(Ok(ExchangeIdResult {
                        client_id: 1.into(),
                        sequence_id: 2.into(),
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
                    })),
                    NfsResOp::CreateSession(Ok(CreateSessionResult {
                        session_id: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16].into(),
                        sequence_id: 2.into(),
                        flags: CreateSessionFlags::CONN_BACK_CHAN,
                        fore_channel_attributes: ChannelAttributes {
                            header_pad_size: 1,
                            max_request_size: 2,
                            max_response_size: 3,
                            max_response_size_cached: 4,
                            max_operations: 5,
                            max_requests: 6,
                            rdma_ird: Some(7),
                        },
                        back_channel_attributes: ChannelAttributes {
                            header_pad_size: 8,
                            max_request_size: 9,
                            max_response_size: 10,
                            max_response_size_cached: 11,
                            max_operations: 12,
                            max_requests: 13,
                            rdma_ird: Some(14),
                        },
                    })),
                    NfsResOp::DestroySession(Err(Error::PERM)),
                    NfsResOp::Sequence(Ok(SequenceResult {
                        session_id: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16].into(),
                        sequence_id: 2.into(),
                        slot_id: 3.into(),
                        highest_slot_id: 4.into(),
                        target_highest_slot_id: 5.into(),
                        status_flags: SequenceStatusFlags::EXPIRED_SOME_STATE_REVOKED
                            | SequenceStatusFlags::LEASE_MOVED,
                    })),
                    NfsResOp::ReclaimComplete(Err(Error::DENIED)),
                ],
            })),
        });
        let mut buffer = [0u8; 10240];
        let buffer = serialize!(
            tuple((encode::message(message.clone()), encode::reply(&reply),)),
            buffer
        );
        let (buffer, decoded_message) = decode::message(buffer).unwrap();
        let (buffer, decoded_reply) = decode::reply(ProcedureNumber::Compound)(buffer).unwrap();
        assert_eq!(message, decoded_message);
        assert_eq!(reply, decoded_reply);
    }
}

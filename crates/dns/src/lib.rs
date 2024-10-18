#[cfg(feature = "client")]
mod client;
mod error;
mod protocol;
#[cfg(feature = "server")]
mod server;

pub use self::error::{Error, Result};
#[cfg(feature = "client")]
pub use client::Client;
#[cfg(feature = "server")]
pub use server::{Handler, Server};

#[derive(Clone, Debug)]
pub struct Query {
    pub transaction_id: u16,
    pub operation_code: OperationCode,
    pub recursion_desired: bool,
    pub name: String,
    pub record_type: RecordType,
    pub class: Class,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub transaction_id: u16,
    pub operation_code: OperationCode,
    pub authoritative: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,
    pub response_code: ResponseCode,
    pub answers: Vec<Answer>,
}

#[derive(Debug, Clone)]
pub struct Answer {
    pub name: String,
    pub record_type: RecordType,
    pub class: Class,
    pub ttl: u32,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum OperationCode {
    StandardQuery,
    InverseQuery,
    Status,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum RecordType {
    A,
    Aaaa,
    Cname,
    Mx,
    Ns,
    Ptr,
    Soa,
    Srv,
    Txt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Internet,
    Csnet,
    Chaos,
    Hesiod,
    None,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseCode {
    NoError,
    Malformed,
    ServerFailure,
    NonExistentDomain,
    NotImplemented,
    Refused,
}

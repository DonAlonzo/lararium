#[cfg(feature = "proto")]
mod proto;

#[cfg(feature = "proto")]
pub use proto::{
    admittance_client::AdmittanceClient,
    admittance_server::{Admittance, AdmittanceServer},
    gateway_client::GatewayClient,
    gateway_server::{Gateway, GatewayServer},
    library_client::LibraryClient,
    library_server::{Library, LibraryServer},
    DESCRIPTOR_SET,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Clone)]
pub struct ClientInfo {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct JoinRequest {
    pub csr: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(ProposeRequest, Serialize))]
pub struct JoinResponse {
    pub ca: String,
    pub certificate: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct CheckInRequest {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct CheckInResponse {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct CheckOutRequest {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct CheckOutResponse {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct HeartbeatRequest {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct HeartbeatResponse {}

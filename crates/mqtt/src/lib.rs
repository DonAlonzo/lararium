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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Connect {
    pub clean_start: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Connack {
    pub reason_code: ConnectReasonCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Publish<'a> {
    pub topic_name: &'a str,
    pub payload: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Puback {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subscribe<'a> {
    pub topic_name: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Suback<'a> {
    pub reason_codes: &'a [SubscribeReasonCode],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Disconnect {
    pub reason_code: DisconnectReasonCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    V3_1_1,
    V5_0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectReasonCode {
    Success,
    UnspecifiedError,
    MalformedPacket,
    ProtocolError,
    ImplementationSpecificError,
    UnsupportedProtocolVersion,
    ClientIdentifierNotValid,
    BadUserNameOrPassword,
    NotAuthorized,
    ServerUnavailable,
    ServerBusy,
    Banned,
    BadAuthenticationMethod,
    TopicNameInvalid,
    PacketTooLarge,
    QuotaExceeded,
    PayloadFormatInvalid,
    RetainNotSupported,
    QoSNotSupported,
    UseAnotherServer,
    ServerMoved,
    ConnectionRateExceeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PubackReasonCode {
    Success,
    NoMatchingSubscribers,
    UnspecifiedError,
    ImplementationSpecificError,
    NotAuthorized,
    TopicNameInvalid,
    PacketIdentifierInUse,
    QuotaExceeded,
    PayloadFormatInvalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscribeReasonCode {
    GrantedQoS0,
    GrantedQoS1,
    GrantedQoS2,
    UnspecifiedError,
    ImplementationSpecificError,
    NotAuthorized,
    TopicFilterInvalid,
    PacketIdentifierInUse,
    QuotaExceeded,
    SharedSubscriptionsNotSupported,
    SubscriptionIdentifiersNotSupported,
    WildcardSubscriptionsNotSupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisconnectReasonCode {
    NormalDisconnection,
    DisconnectWithWillMessage,
    UnspecifiedError,
    MalformedPacket,
    ProtocolError,
    ImplementationSpecificError,
    NotAuthorized,
    ServerBusy,
    ServerShuttingDown,
    KeepAliveTimeout,
    SessionTakenOver,
    TopicFilterInvalid,
    TopicNameInvalid,
    ReceiveMaximumExceeded,
    TopicAliasInvalid,
    PacketTooLarge,
    MessageRateTooHigh,
    QuotaExceeded,
    AdministrativeAction,
    PayloadFormatInvalid,
    RetainNotSupported,
    QoSNotSupported,
    UseAnotherServer,
    ServerMoved,
    SharedSubscriptionsNotSupported,
    ConnectionRateExceeded,
    MaximumConnectTime,
    SubscriptionIdentifiersNotSupported,
    WildcardSubscriptionsNotSupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QoS {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

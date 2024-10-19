#[cfg(feature = "client")]
mod client;
mod error;
mod protocol;
#[cfg(feature = "server")]
mod server;

pub use self::error::{Error, Result};
#[cfg(feature = "client")]
pub use client::*;
#[cfg(feature = "server")]
pub use server::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Message {
    message_type: MessageType,
    hardware_type: HardwareType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MessageType {
    BOOTREQUEST,
    BOOTREPLY,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HardwareType {
    Ethernet,
}

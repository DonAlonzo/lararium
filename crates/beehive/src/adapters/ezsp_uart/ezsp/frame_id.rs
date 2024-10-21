use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameId {
    Version = 0x0000,
    NetworkInit = 0x0017,
    StackStatusHandler = 0x0019,
    FormNetwork = 0x001E,
    UnknownCommand = 0x0058,
    SetInitialSecurityState = 0x0068,
}

impl Into<u16> for FrameId {
    fn into(self) -> u16 {
        self as u16
    }
}

impl TryFrom<u16> for FrameId {
    type Error = u16;

    fn try_from(x: u16) -> Result<Self, Self::Error> {
        use FrameId::*;
        Ok(match x {
            x if x == Version as u16 => Version,
            x if x == NetworkInit as u16 => NetworkInit,
            x if x == StackStatusHandler as u16 => StackStatusHandler,
            x if x == FormNetwork as u16 => FormNetwork,
            x if x == UnknownCommand as u16 => UnknownCommand,
            x if x == SetInitialSecurityState as u16 => SetInitialSecurityState,
            _ => return Err(x),
        })
    }
}

impl TryFrom<u8> for FrameId {
    type Error = u16;

    fn try_from(x: u8) -> Result<Self, Self::Error> {
        FrameId::try_from(x as u16)
    }
}

impl Display for FrameId {
    fn fmt(
        &self,
        f: &mut Formatter,
    ) -> std::fmt::Result {
        write!(f, "{:02X}", *self as u16)
    }
}

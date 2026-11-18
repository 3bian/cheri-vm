// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod reason;
mod reply_error;
mod role;

pub(crate) use reason::Reason;
pub(crate) use reply_error::ReplyError;
pub(crate) use role::Role;

use serde::{Deserialize, Serialize};

use super::endpoint::Kind;

use super::{MAX_BURST_BYTES, MAX_MESSAGE_BYTES};

use crate::websocket::base64;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum Message {
    Hello {
        format_version: u32,
        profile: String,
        device: String,
        kind: Kind,
        write_only: bool,
    },
    Ready {},
    Uart {
        data: String,
    },
    Gpio {
        levels: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        direction: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mask: Option<u32>,
    },
    Spi {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        selected: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mode: Option<u8>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<u64>,
    },
    I2c {
        id: u64,
        write: String,
        read: u32,
    },
    Ethernet {
        frame: String,
    },
    Sync {
        id: u64,
    },
    Reply {
        id: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        error: Option<ReplyError>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    Bye {
        reason: Reason,
        message: String,
    },
    Error {
        message: String,
    },
}

impl Message {
    pub(crate) fn check(&self, from: Role) -> Result<(), String> {
        let machine = from == Role::Machine;
        let allowed = match self {
            Self::Error { .. }
            | Self::Uart { .. }
            | Self::Gpio { .. }
            | Self::Ethernet { .. } => true,
            Self::Hello { .. }
            | Self::Bye { .. }
            | Self::Spi { .. }
            | Self::I2c { .. }
            | Self::Sync { .. } => machine,
            Self::Reply { .. } | Self::Ready {} => !machine,
        };
        if !allowed {
            return Err(format!("{} is not sent by this role", self.name()));
        }
        match self {
            Self::Uart { data, .. } => bytes(data, "data")?,
            Self::Ethernet { frame, .. } => bytes(frame, "frame")?,
            Self::Gpio {
                direction, mask, ..
            } => {
                if machine != direction.is_some() || machine == mask.is_some() {
                    return Err("gpio carries direction from the machine and \
                                mask from a device"
                        .into());
                }
            }
            Self::Spi {
                selected,
                mode,
                data,
                id,
                ..
            } => match (selected, mode, data) {
                (Some(_), None, None) if id.is_none() => {}
                (None, Some(mode), Some(data))
                    if *mode <= 3 && *id != Some(0) =>
                {
                    bytes(data, "data")?;
                }
                _ => {
                    return Err("spi carries selected, or mode 0 to 3 and \
                                data with an optional id"
                        .into());
                }
            },
            Self::I2c {
                id, write, read, ..
            } => {
                bytes(write, "write")?;
                if *id == 0 || *read as usize > MAX_BURST_BYTES {
                    return Err("i2c needs a positive id and reads at most \
                                4096 bytes"
                        .into());
                }
            }
            Self::Sync { id } => {
                if *id == 0 {
                    return Err("sync needs a positive id".into());
                }
            }
            Self::Reply {
                data,
                error,
                message,
                ..
            } => {
                if data.is_some() == error.is_some()
                    || (message.is_some() && error.is_none())
                {
                    return Err("reply carries data, or error with an \
                                optional message"
                        .into());
                }
                if let Some(data) = data {
                    bytes(data, "data")?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::Hello { .. } => "hello",
            Self::Ready {} => "ready",
            Self::Uart { .. } => "uart",
            Self::Gpio { .. } => "gpio",
            Self::Spi { .. } => "spi",
            Self::I2c { .. } => "i2c",
            Self::Ethernet { .. } => "ethernet",
            Self::Sync { .. } => "sync",
            Self::Reply { .. } => "reply",
            Self::Bye { .. } => "bye",
            Self::Error { .. } => "error",
        }
    }

    pub(crate) fn parse(text: &str, from: Role) -> Result<Self, String> {
        if text.len() > MAX_MESSAGE_BYTES {
            return Err("a message is at most 1 MiB".into());
        }
        let message: Self = serde_json::from_str(text)
            .map_err(|error| format!("not a message: {error}"))?;
        message.check(from)?;
        Ok(message)
    }

    pub(crate) fn text(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

fn bytes(text: &str, field: &str) -> Result<(), String> {
    base64::is_canonical(text)
        .then_some(())
        .ok_or_else(|| format!("{field} is not base64"))
}

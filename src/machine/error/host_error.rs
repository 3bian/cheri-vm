// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::HostErrorKind;

#[derive(Debug)]
pub struct HostError {
    pub(super) kind: HostErrorKind,
    pub(super) message: String,
}

impl HostError {
    pub const fn kind(&self) -> HostErrorKind {
        self.kind
    }

    pub fn new(kind: HostErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn other(message: impl Into<String>) -> Self {
        Self::new(HostErrorKind::Other, message)
    }
}

impl core::error::Error for HostError {}

impl core::fmt::Display for HostError {
    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl From<std::io::Error> for HostError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.kind().into(), error.to_string())
    }
}

impl From<std::io::ErrorKind> for HostError {
    fn from(kind: std::io::ErrorKind) -> Self {
        std::io::Error::from(kind).into()
    }
}

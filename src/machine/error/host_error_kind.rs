// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostErrorKind {
    BrokenPipe,
    ConnectionAborted,
    ConnectionReset,
    Interrupted,
    InvalidInput,
    NotConnected,
    Other,
    PermissionDenied,
    TimedOut,
    UnexpectedEof,
    Unsupported,
    WouldBlock,
    WriteZero,
}

impl From<std::io::ErrorKind> for HostErrorKind {
    fn from(kind: std::io::ErrorKind) -> Self {
        use std::io::ErrorKind;
        match kind {
            ErrorKind::BrokenPipe => Self::BrokenPipe,
            ErrorKind::ConnectionAborted => Self::ConnectionAborted,
            ErrorKind::ConnectionReset => Self::ConnectionReset,
            ErrorKind::Interrupted => Self::Interrupted,
            ErrorKind::InvalidInput => Self::InvalidInput,
            ErrorKind::NotConnected => Self::NotConnected,
            ErrorKind::PermissionDenied => Self::PermissionDenied,
            ErrorKind::TimedOut => Self::TimedOut,
            ErrorKind::UnexpectedEof => Self::UnexpectedEof,
            ErrorKind::Unsupported => Self::Unsupported,
            ErrorKind::WouldBlock => Self::WouldBlock,
            ErrorKind::WriteZero => Self::WriteZero,
            _ => Self::Other,
        }
    }
}

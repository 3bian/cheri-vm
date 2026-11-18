// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::HostError;
use super::exception_cause::ExceptionCause;

#[derive(Debug)]
pub enum EmulatorError {
    Bus {
        address: u64,
        explanation: String,
    },
    CheriotTrap {
        address: u64,
        cause: u32,
        value: u64,
    },
    Configuration(String),
    Execution(String),
    Firmware(String),
    HostIo(HostError),
    Interrupt {
        address: u64,
        cause: u32,
    },
    Refused(String),
    Trap {
        cause: ExceptionCause,
        address: u64,
        value: u64,
    },
}

impl EmulatorError {
    pub(crate) fn host_file(
        path: impl AsRef<std::path::Path>,
        error: &std::io::Error,
    ) -> Self {
        Self::HostIo(HostError::new(
            error.kind().into(),
            format!("{}: {error}", path.as_ref().display()),
        ))
    }
}

impl core::error::Error for EmulatorError {}

impl core::fmt::Display for EmulatorError {
    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        match self {
            Self::Bus {
                address,
                explanation,
            } => {
                write!(formatter, "bus error at 0x{address:08x}: {explanation}")
            }
            Self::CheriotTrap {
                address,
                cause,
                value,
            } => write!(
                formatter,
                "CHERIoT trap cause {cause:#x} at 0x{address:08x} (value \
                 0x{value:08x})"
            ),
            Self::Configuration(message) => {
                write!(formatter, "configuration error: {message}")
            }
            Self::Execution(message) => {
                write!(formatter, "execution error: {message}")
            }
            Self::Firmware(message) => {
                write!(formatter, "firmware error: {message}")
            }
            Self::HostIo(error) => write!(formatter, "host I/O error: {error}"),
            Self::Interrupt { address, cause } => write!(
                formatter,
                "unhandled machine interrupt cause {cause} at 0x{address:08x}"
            ),
            Self::Refused(message) => write!(
                formatter,
                "the guest may not run this instruction: {message}"
            ),
            Self::Trap {
                cause,
                address,
                value,
            } => write!(
                formatter,
                "processor exception {cause:?} at 0x{address:08x} (value \
                 0x{value:08x})"
            ),
        }
    }
}

impl From<HostError> for EmulatorError {
    fn from(error: HostError) -> Self {
        Self::HostIo(error)
    }
}

impl From<std::io::Error> for EmulatorError {
    fn from(error: std::io::Error) -> Self {
        Self::HostIo(error.into())
    }
}

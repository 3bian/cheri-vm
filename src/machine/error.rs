// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod emulator_error;
mod host_error;
mod host_error_kind;

pub use emulator_error::EmulatorError;
pub(crate) use host_error::HostError;
pub(crate) use host_error_kind::HostErrorKind;

mod exception_cause;

pub use exception_cause::ExceptionCause;

pub(crate) type HostResult<T> = Result<T, HostError>;

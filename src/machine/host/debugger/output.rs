// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod debug_output;
mod delivery;

pub(crate) use debug_output::DebugOutput;
pub(crate) use delivery::Delivery;

use std::os::fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd};

use nix::fcntl::{FcntlArg, OFlag, fcntl};

use crate::machine::host::open_flags::non_blocking;

fn reopen(original: BorrowedFd) -> Option<OwnedFd> {
    let is_non_blocking = |descriptor: BorrowedFd| {
        let flags = fcntl(descriptor, FcntlArg::F_GETFL).ok()?;
        Some(OFlag::from_bits_retain(flags).contains(OFlag::O_NONBLOCK))
    };
    if is_non_blocking(original)? {
        return None;
    }
    let reopened: OwnedFd =
        non_blocking(std::fs::OpenOptions::new().write(true))
            .open(format!("/proc/self/fd/{}", original.as_raw_fd()))
            .ok()?
            .into();
    (is_non_blocking(reopened.as_fd())? && !is_non_blocking(original)?)
        .then_some(reopened)
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{Delivery, reopen};
use crate::runtime::host_output::debugger_escape;
use nix::errno::Errno;
use nix::poll::{PollFd, PollFlags, PollTimeout, poll};
use nix::sys::socket::{MsgFlags, send};
use nix::sys::stat::{SFlag, fstat, stat};
use nix::unistd::{dup, isatty, write};
use std::collections::VecDeque;
use std::io;
use std::os::fd::{AsFd, AsRawFd, OwnedFd};

pub(crate) struct DebugOutput {
    pub(crate) delivery: Delivery,
    pub(crate) descriptor: OwnedFd,
    pub(crate) pending: VecDeque<u8>,
}

impl DebugOutput {
    pub(crate) const CAPACITY: usize = 64 * 1024;
    pub(super) const POLLED_WRITE_BYTES: usize = 512;

    pub(super) fn can_write(&self) -> io::Result<bool> {
        let mut descriptor =
            [PollFd::new(self.descriptor.as_fd(), PollFlags::POLLOUT)];
        match poll(&mut descriptor, PollTimeout::ZERO) {
            Ok(_) => Ok(descriptor[0]
                .revents()
                .is_some_and(|events| !events.is_empty())),
            Err(Errno::EINTR) => Ok(false),
            Err(error) => Err(error.into()),
        }
    }

    pub(crate) fn enqueue(
        &mut self,
        bytes: &[u8],
        raw: bool,
    ) -> io::Result<()> {
        let length = bytes
            .iter()
            .map(|byte| if raw { 1 } else { debugger_escape(*byte).1 })
            .sum::<usize>();
        if length > Self::CAPACITY - self.pending.len() {
            return Err(io::Error::other(
                "debugger host output budget exhausted",
            ));
        }
        for byte in bytes {
            if raw {
                self.pending.push_back(*byte);
            } else {
                let (escaped, length) = debugger_escape(*byte);
                self.pending.extend(escaped[..length].iter().copied());
            }
        }
        Ok(())
    }

    pub(crate) fn new(descriptor: impl AsFd) -> io::Result<Self> {
        let original = descriptor.as_fd();
        let metadata = fstat(original)?;
        let kind = SFlag::from_bits_truncate(metadata.st_mode);
        let null_device = kind == SFlag::S_IFCHR
            && stat("/dev/null")
                .is_ok_and(|null| null.st_rdev == metadata.st_rdev);
        if !matches!(kind, SFlag::S_IFIFO | SFlag::S_IFSOCK)
            && !null_device
            && !isatty(original).unwrap_or(false)
        {
            return Err(io::Error::other(
                "debugger output requires a pipe, terminal or /dev/null; pipe \
                 through tee for file capture",
            ));
        }
        let (descriptor, delivery) = if kind == SFlag::S_IFSOCK {
            (dup(original)?, Delivery::Socket)
        } else if let Some(reopened) = reopen(original) {
            (reopened, Delivery::NonBlocking)
        } else {
            (dup(original)?, Delivery::Polled)
        };
        Ok(Self {
            delivery,
            descriptor,
            pending: VecDeque::new(),
        })
    }

    pub(crate) fn pending(&self) -> usize {
        self.pending.len()
    }

    pub(crate) fn pump(&mut self) -> io::Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let bytes = self.pending.as_slices().0;
        let result = match self.delivery {
            Delivery::NonBlocking => write(&self.descriptor, bytes),
            Delivery::Polled => {
                if !self.can_write()? {
                    return Ok(());
                }
                let length = bytes.len().min(Self::POLLED_WRITE_BYTES);
                write(&self.descriptor, &bytes[..length])
            }
            Delivery::Socket => send(
                self.descriptor.as_raw_fd(),
                bytes,
                MsgFlags::MSG_DONTWAIT
                    | MsgFlags::from_bits_retain(nix::libc::MSG_NOSIGNAL),
            ),
        };
        match result {
            Ok(0) => Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "debug output closed",
            )),
            Ok(count) => {
                self.pending.drain(..count);
                Ok(())
            }
            Err(Errno::EAGAIN | Errno::EINTR) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

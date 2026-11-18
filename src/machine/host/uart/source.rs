// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::io::{self, Read};
use std::path::Path;

use crate::machine::EmulatorError;

#[cfg(unix)]
pub(super) struct Source {
    file: std::fs::File,
    terminal: bool,
}

#[cfg(not(unix))]
pub(super) struct Source;

#[cfg(unix)]
impl Source {
    pub(super) fn hung_up(&self) -> io::Result<bool> {
        use std::os::fd::AsFd;

        use nix::errno::Errno;
        use nix::poll::{PollFd, PollFlags, PollTimeout, poll};

        if !self.terminal {
            return Ok(true);
        }
        let mut descriptor =
            [PollFd::new(self.file.as_fd(), PollFlags::POLLIN)];
        match poll(&mut descriptor, PollTimeout::ZERO) {
            Ok(_) => Ok(descriptor[0]
                .revents()
                .is_some_and(|events| events.contains(PollFlags::POLLHUP))),
            Err(Errno::EINTR) => Ok(false),
            Err(error) => Err(error.into()),
        }
    }

    pub(super) fn open(path: &Path) -> Result<Self, EmulatorError> {
        use crate::machine::host::open_flags::non_blocking;

        let file = non_blocking(std::fs::OpenOptions::new().read(true))
            .open(path)
            .map_err(|error| EmulatorError::host_file(path, &error))?;
        if file.metadata()?.is_dir() {
            return Err(EmulatorError::Configuration(
                "UART input must be a readable stream".into(),
            ));
        }
        let terminal = nix::unistd::isatty(&file).map_err(io::Error::from)?;
        Ok(Self { file, terminal })
    }
}

#[cfg(not(unix))]
impl Source {
    pub(super) fn hung_up(&self) -> io::Result<bool> {
        Ok(true)
    }

    pub(super) fn open(path: &Path) -> Result<Self, EmulatorError> {
        Err(EmulatorError::Configuration(format!(
            "UART input from {} needs a Unix system: this one cannot read a \
             host stream without waiting",
            path.display()
        )))
    }
}

#[cfg(unix)]
impl Read for Source {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.file.read(bytes)
    }
}

#[cfg(not(unix))]
impl Read for Source {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }
}

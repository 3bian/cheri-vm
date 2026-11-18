// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::fs::OpenOptions;

pub(crate) fn non_blocking(options: &mut OpenOptions) -> &mut OpenOptions {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(nix::libc::O_NONBLOCK | nix::libc::O_NOCTTY);
    }
    options
}

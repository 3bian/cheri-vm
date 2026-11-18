// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

pub(super) enum Run {
    Continue,
    Line {
        start: Option<(PathBuf, u64)>,
        function: Option<(u64, u64)>,
        over: bool,
        budget: u64,
    },
    Out {
        target: u64,
        budget: u64,
    },
}

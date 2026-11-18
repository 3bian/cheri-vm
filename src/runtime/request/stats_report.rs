// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct StatsReport {
    pub file: Option<PathBuf>,
    pub format: crate::program::output::Format,
}

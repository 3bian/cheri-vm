// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::RunArgs;
use cheri_vm::program::arguments::Parsed;
use cheri_vm::program::options::QUIET;

pub(crate) struct RunCommandArgs {
    pub quiet: bool,
    pub run: RunArgs,
}

impl RunCommandArgs {
    pub(crate) fn from(parsed: &Parsed) -> Self {
        Self {
            quiet: parsed.has(QUIET.long),
            run: RunArgs::from(parsed),
        }
    }
}

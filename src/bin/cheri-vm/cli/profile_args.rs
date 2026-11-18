// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::LIST;
use cheri_vm::program::arguments::Parsed;
use cheri_vm::program::options::{FORMAT, OUTPUT};
use cheri_vm::program::output::Format;
use std::path::PathBuf;

pub(crate) struct ProfileArgs {
    pub format: Option<Format>,
    pub list: bool,
    pub name: Option<String>,
    pub output: Option<PathBuf>,
}

impl ProfileArgs {
    pub(crate) fn from(parsed: &Parsed) -> Self {
        Self {
            format: parsed.one(FORMAT.long).and_then(Format::named),
            list: parsed.has(LIST.long),
            name: parsed
                .positionals
                .first()
                .map(|name| name.to_string_lossy().into_owned()),
            output: parsed.one(OUTPUT.long).map(PathBuf::from),
        }
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{Flag, Positional};

pub struct CommandSpec {
    pub name: &'static str,
    pub options: &'static [Flag],
    pub positionals: &'static [Positional],
    pub summary: &'static str,
}

impl CommandSpec {
    pub fn flag(&self, long: &str) -> Option<&'static Flag> {
        self.options.iter().find(|flag| flag.long == long)
    }
}

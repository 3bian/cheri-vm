// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::CommandSpec;
use std::collections::BTreeMap;
use std::ffi::OsString;

pub struct Parsed {
    pub command: Option<&'static CommandSpec>,
    pub positionals: Vec<OsString>,
    pub(super) values: BTreeMap<&'static str, Vec<String>>,
}

impl Parsed {
    pub fn all(&self, long: &str) -> Vec<String> {
        self.values.get(long).cloned().unwrap_or_default()
    }

    pub fn has(&self, long: &str) -> bool {
        self.values.contains_key(long)
    }

    pub fn number(&self, long: &str) -> Option<u64> {
        self.one(long)?.parse().ok()
    }

    pub fn one(&self, long: &str) -> Option<&str> {
        match self.values.get(long).and_then(|values| values.last()) {
            Some(value) => Some(value),
            None => self
                .command
                .and_then(|command| command.flag(long))
                .and_then(|flag| flag.default),
        }
    }
}

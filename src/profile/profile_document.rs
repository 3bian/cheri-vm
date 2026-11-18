// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::program::diagnostic::{Diagnostic, Result};
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct ProfileDocument {
    pub description: Option<String>,
    pub file: PathBuf,
    pub name: String,
    pub table: toml::Table,
}

impl ProfileDocument {
    pub fn has_machine(&self) -> bool {
        self.table.contains_key("emulator")
    }

    pub fn to_toml(&self) -> Result<String> {
        toml::to_string(&self.table).map_err(|error| {
            Diagnostic::new(format!("profile {}: {error}", self.name))
        })
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{default_memory_kind, executable};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MemoryEntry {
    pub(super) start: u64,
    pub(super) length: usize,
    #[serde(default = "default_memory_kind", rename = "type")]
    pub(super) kind: String,
    #[serde(default = "executable")]
    pub(super) executable: bool,
    pub(super) tagged: Option<bool>,
}

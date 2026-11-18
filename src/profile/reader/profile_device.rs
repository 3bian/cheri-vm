// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProfileDevice {
    pub(super) start: u64,
    #[serde(rename = "end")]
    pub(super) _end: Option<u64>,
    #[serde(rename = "length")]
    pub(super) _length: Option<u64>,
    pub(super) model: String,
    pub(super) interrupt: Option<String>,
    pub(super) interrupt_source: Option<u8>,
    pub(super) attached_device: Option<String>,
    pub(super) sectors: Option<usize>,
    pub(super) read_only: Option<bool>,
}

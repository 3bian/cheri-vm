// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{Completion, Timing, default_steps_per_tick};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EmulatorSettings {
    pub(super) processor: String,
    pub(super) extensions: Option<Vec<String>>,
    #[serde(default)]
    pub(super) completion: Option<Completion>,
    #[serde(default = "default_steps_per_tick")]
    pub(super) steps_per_tick: u32,
    #[serde(default)]
    pub(super) timing: Timing,
    pub(super) timer_hz: Option<u32>,
    #[serde(default)]
    pub(super) memory: toml::Table,
}

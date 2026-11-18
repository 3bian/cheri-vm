// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::host::gpio::settings::GpioSettings;

pub(crate) struct GpioChip {
    pub lines: Vec<u32>,
    pub path: String,
    pub settings: GpioSettings,
}

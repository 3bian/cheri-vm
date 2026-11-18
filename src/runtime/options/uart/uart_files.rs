// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::host::uart::settings::UartSettings;

pub(crate) struct UartFiles {
    pub input: Option<String>,
    pub output: Option<String>,
    pub settings: Option<UartSettings>,
}

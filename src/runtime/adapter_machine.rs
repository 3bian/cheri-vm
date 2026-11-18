// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::debug::adapter::System;

pub struct AdapterMachine {
    pub system: Box<dyn System>,
    pub firmware: Vec<u8>,
    pub(super) _uart: crate::runtime::uart::connections::UartConnections,
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::SpiPeripheral;
use crate::machine::error::HostResult;

#[derive(Default)]
pub(crate) struct LoopbackPeripheral;

impl SpiPeripheral for LoopbackPeripheral {
    fn transfer(&mut self, outgoing: u8, _mode: u8) -> HostResult<u8> {
        Ok(outgoing)
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::error::HostResult;

pub(crate) trait SpiPeripheral {
    fn select(&mut self, _selected: bool) -> HostResult<()> {
        Ok(())
    }

    fn transfer(&mut self, outgoing: u8, mode: u8) -> HostResult<u8>;
}

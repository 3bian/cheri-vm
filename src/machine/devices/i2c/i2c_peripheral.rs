// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub(crate) trait I2cPeripheral {
    fn read_byte(&mut self) -> u8;

    fn start(&mut self, _read: bool) -> bool {
        true
    }

    fn stop(&mut self) {}

    fn write_byte(&mut self, value: u8) -> bool;
}

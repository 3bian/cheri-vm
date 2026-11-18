// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Device;

pub(crate) struct DeviceMapping {
    pub base: u64,
    pub device: Box<dyn Device>,
    pub interrupt_source: Option<u8>,
    pub name: String,
    pub size: u32,
}

impl DeviceMapping {
    pub(crate) fn contains(&self, address: u64, width: usize) -> bool {
        address >= self.base
            && (width as u64) <= u64::from(self.size)
            && address - self.base <= u64::from(self.size) - width as u64
    }
}

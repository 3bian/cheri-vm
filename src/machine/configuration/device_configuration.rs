// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::devices::uart::UartLayout;

#[derive(Debug, Clone)]
pub struct DeviceConfiguration {
    pub attached_device: String,
    pub base: u64,
    pub interrupt_source: Option<u8>,
    pub kind: String,
    pub layout: Option<UartLayout>,
    pub name: String,
    pub read_only: bool,
    pub sectors: usize,
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Kind {
    Ethernet,
    Gpio,
    I2c,
    Spi,
    Uart,
}

pub(crate) fn i2c(device: &str, address: u8) -> String {
    format!("{device}/0x{address:02x}")
}

pub(crate) fn spi(device: &str, select: u32) -> String {
    format!("{device}/{select}")
}

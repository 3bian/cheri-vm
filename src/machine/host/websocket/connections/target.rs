// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub(crate) enum Target {
    Ethernet,
    Gpio { pins: u32 },
    I2c { addresses: Vec<u8> },
    Spi { select: u32, write_only: bool },
    Uart,
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub(crate) mod connections;
#[cfg(all(feature = "gdb", unix))]
pub(crate) mod debugger;
pub(crate) mod device_path;
pub(crate) mod device_selection;
pub(crate) mod ethernet;
pub(crate) mod gpio;
pub(crate) mod i2c;
pub(crate) mod open_flags;
pub(crate) mod spi;
pub(crate) mod uart;
pub(crate) mod websocket;

pub(crate) use connections::HostConnections;
pub(crate) use i2c::connection::I2cConnection;
pub(crate) use spi::connection::SpiConnection;

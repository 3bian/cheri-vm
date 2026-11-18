// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub(crate) mod connection;
pub(crate) mod connections;
mod ethernet;
mod gpio;
mod i2c;
mod link;
mod socket;
mod spi;
pub(crate) mod uart;

pub(crate) use connections::{
    Address, Served, Target, WebSocketBinding, WebSocketConnections,
};

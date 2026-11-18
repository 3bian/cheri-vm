// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{Address, Target};
use crate::websocket::endpoint::{self, Kind};
use std::time::Duration;

pub(crate) struct WebSocketBinding {
    pub address: Address,
    pub device: String,
    pub target: Target,
    pub timeout: Duration,
}

impl WebSocketBinding {
    pub(super) fn kind(&self) -> (Kind, bool) {
        match &self.target {
            Target::Ethernet => (Kind::Ethernet, false),
            Target::Gpio { .. } => (Kind::Gpio, false),
            Target::I2c { .. } => (Kind::I2c, false),
            Target::Spi { write_only, .. } => (Kind::Spi, *write_only),
            Target::Uart => (Kind::Uart, false),
        }
    }

    pub(crate) fn names(&self) -> Vec<String> {
        match &self.target {
            Target::I2c { addresses } => addresses
                .iter()
                .map(|address| endpoint::i2c(&self.device, *address))
                .collect(),
            Target::Spi { select, .. } => {
                vec![endpoint::spi(&self.device, *select)]
            }
            _ => vec![self.device.clone()],
        }
    }
}

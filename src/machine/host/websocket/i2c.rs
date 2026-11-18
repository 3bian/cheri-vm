// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::io;

use super::link::Link;
use crate::machine::devices::i2c::I2cTransport;
use crate::websocket::base64;
use crate::websocket::endpoint;
use crate::websocket::message::Message;

pub(super) struct WebSocketI2c {
    pub(super) device: String,
    pub(super) links: Vec<(u8, Link)>,
}

impl I2cTransport for WebSocketI2c {
    fn transfer(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> crate::machine::error::HostResult<()> {
        let endpoint = endpoint::i2c(&self.device, address);
        let link = self
            .links
            .iter()
            .find(|(bound, _)| *bound == address)
            .map(|(_, link)| link)
            .ok_or_else(|| {
                io::Error::other(format!("{endpoint} has no host"))
            })?;
        let data = link.request(|id| Message::I2c {
            id,
            write: base64::encode(write),
            read: read.len() as u32,
        })?;
        if data.len() != read.len() {
            return Err(io::Error::other(format!(
                "{endpoint} answered {} bytes to a read of {}",
                data.len(),
                read.len()
            ))
            .into());
        }
        read.copy_from_slice(&data);
        Ok(())
    }
}

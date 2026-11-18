// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::io;

use super::link::Link;
use crate::machine::devices::spi::SpiPeripheral;
use crate::websocket::base64;
use crate::websocket::message::Message;

pub(super) struct WebSocketSpi {
    pub(super) link: Link,
    pub(super) write_only: bool,
}

impl SpiPeripheral for WebSocketSpi {
    fn select(
        &mut self,
        selected: bool,
    ) -> crate::machine::error::HostResult<()> {
        Ok(self.link.send(&Message::Spi {
            selected: Some(selected),
            mode: None,
            data: None,
            id: None,
        })?)
    }

    fn transfer(
        &mut self,
        outgoing: u8,
        mode: u8,
    ) -> crate::machine::error::HostResult<u8> {
        if self.write_only {
            self.link.buffer(mode, outgoing)?;
            return Ok(0xff);
        }
        let endpoint = self.link.device();
        let data = self.link.request(|id| Message::Spi {
            selected: None,
            mode: Some(mode),
            data: Some(base64::encode(&[outgoing])),
            id: Some(id),
        })?;
        match data.as_slice() {
            [byte] => Ok(*byte),
            _ => Err(io::Error::other(format!(
                "{endpoint} answered {} bytes to one",
                data.len()
            ))
            .into()),
        }
    }
}

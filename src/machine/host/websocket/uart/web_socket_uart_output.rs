// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::link::Link;
use crate::websocket::MAX_BURST_BYTES;
use crate::websocket::base64;
use crate::websocket::message::Message;
use std::io;

pub(crate) struct WebSocketUartOutput {
    pub(crate) device: String,
    pub(in crate::machine::host::websocket) link: Link,
}

impl WebSocketUartOutput {
    pub(crate) fn write(&self, bytes: &[u8]) -> io::Result<()> {
        for chunk in bytes.chunks(MAX_BURST_BYTES) {
            self.link.send(&Message::Uart {
                data: base64::encode(chunk),
            })?;
        }
        Ok(())
    }
}

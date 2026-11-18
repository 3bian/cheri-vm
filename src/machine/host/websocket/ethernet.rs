// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::io;

use super::link::Link;
use crate::machine::devices::ethernet::FrameTransport;
use crate::websocket::base64;
use crate::websocket::message::Message;

pub(super) struct WebSocketFrames {
    pub(super) link: Link,
}

impl FrameTransport for WebSocketFrames {
    fn receive(
        &mut self,
        frame: &mut [u8],
    ) -> crate::machine::error::HostResult<usize> {
        let Some(Message::Ethernet { frame: text, .. }) = self.link.take()?
        else {
            return Err(io::ErrorKind::WouldBlock.into());
        };
        let bytes = base64::decode(&text).unwrap_or_default();
        let length = bytes.len().min(frame.len());
        frame[..length].copy_from_slice(&bytes[..length]);
        Ok(length)
    }

    fn transmit(
        &mut self,
        frame: &[u8],
    ) -> crate::machine::error::HostResult<usize> {
        self.link.send(&Message::Ethernet {
            frame: base64::encode(frame),
        })?;
        Ok(frame.len())
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Burst;
use crate::websocket::MAX_BURST_BYTES;
use crate::websocket::base64;
use crate::websocket::message::Message;

#[derive(Default)]
pub(crate) struct Bursts {
    pub(super) held: Option<Burst>,
    pub(super) seen: usize,
}

impl Bursts {
    pub(crate) fn fits(&self, mode: u8) -> bool {
        self.held.as_ref().is_none_or(|burst| burst.mode == mode)
    }

    pub(crate) fn push(&mut self, mode: u8, byte: u8) -> bool {
        let burst = self.held.get_or_insert_with(|| Burst {
            bytes: Vec::new(),
            mode,
        });
        burst.bytes.push(byte);
        burst.bytes.len() >= MAX_BURST_BYTES
    }

    pub(crate) fn settled(&mut self) -> bool {
        let held = self.held.as_ref().map_or(0, |burst| burst.bytes.len());
        let settled = held != 0 && held == self.seen;
        self.seen = held;
        settled
    }

    pub(crate) fn take(&mut self) -> Option<Message> {
        self.seen = 0;
        let burst = self.held.take()?;
        Some(Message::Spi {
            selected: None,
            mode: Some(burst.mode),
            data: Some(base64::encode(&burst.bytes)),
            id: None,
        })
    }
}

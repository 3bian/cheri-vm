// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::link::Link;
use crate::machine::devices::gpio::LineTransport;
use crate::websocket::message::Message;

pub(super) struct WebSocketLines {
    direction: u32,
    inputs: u32,
    link: Link,
    pins: u32,
    sent: Option<(u32, u32)>,
    watching: bool,
}

impl WebSocketLines {
    pub(super) fn new(link: Link, pins: u32) -> Self {
        Self {
            direction: 0,
            inputs: 0,
            link,
            pins,
            sent: None,
            watching: false,
        }
    }
}

impl LineTransport for WebSocketLines {
    fn read(&mut self) -> crate::machine::error::HostResult<u32> {
        if !std::mem::replace(&mut self.watching, true) {
            return Ok(self.inputs);
        }
        while let Some(message) = self.link.take()? {
            if let Message::Gpio {
                levels,
                mask: Some(mask),
                ..
            } = message
            {
                let mask = mask & self.pins & !self.direction;
                self.inputs = (self.inputs & !mask) | (levels & mask);
            }
        }
        Ok(self.inputs)
    }

    fn write(
        &mut self,
        direction: u32,
        outputs: u32,
    ) -> crate::machine::error::HostResult<()> {
        let direction = direction & self.pins;
        let levels = outputs & direction;
        self.direction = direction;
        if self.sent == Some((levels, direction)) {
            return Ok(());
        }
        self.sent = Some((levels, direction));
        Ok(self.link.send(&Message::Gpio {
            levels,
            direction: Some(direction),
            mask: None,
        })?)
    }
}

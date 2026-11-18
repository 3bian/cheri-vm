// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::link::Link;
use crate::machine::EmulatorError;
use crate::machine::bus::Bus;
use crate::machine::devices::uart::Uart;
use crate::websocket::base64;
use crate::websocket::message::Message;
use std::collections::VecDeque;

pub(crate) struct WebSocketUartInput {
    pub(in crate::machine::host::websocket) device: String,
    pub(in crate::machine::host::websocket) link: Link,
    pub(in crate::machine::host::websocket) pending: VecDeque<u8>,
}

impl WebSocketUartInput {
    pub(crate) fn poll(&mut self, bus: &mut Bus) -> Result<(), EmulatorError> {
        while self.pending.len() < Uart::RECEIVE_CAPACITY {
            let Some(Message::Uart { data, .. }) = self.link.take()? else {
                break;
            };
            self.pending
                .extend(base64::decode(&data).unwrap_or_default());
        }
        if let Some(uart) = bus.device_mut::<Uart>(&self.device) {
            let count = uart.receive_remaining().min(self.pending.len());
            let bytes: Vec<u8> = self.pending.drain(..count).collect();
            uart.inject_receive(&bytes)?;
        }
        Ok(())
    }
}

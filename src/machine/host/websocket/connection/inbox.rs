// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::collections::VecDeque;

use crate::websocket::MAX_MESSAGE_BYTES;
use crate::websocket::message::Message;

const MAX_HELD_BYTES: usize = 16 * MAX_MESSAGE_BYTES;

#[derive(Default)]
pub(crate) struct Inbox {
    held: usize,
    messages: VecDeque<Message>,
}

impl Inbox {
    pub(crate) fn is_full(&self) -> bool {
        self.held > MAX_MESSAGE_BYTES
    }

    pub(crate) fn pop(&mut self) -> Option<Message> {
        let message = self.messages.pop_front()?;
        self.held = self.held.saturating_sub(size(&message));
        Some(message)
    }

    pub(crate) fn push(&mut self, message: Message) -> bool {
        self.held += size(&message);
        self.messages.push_back(message);
        self.held <= MAX_HELD_BYTES
    }
}

fn size(message: &Message) -> usize {
    match message {
        Message::Uart { data, .. } => data.len(),
        Message::Ethernet { frame, .. } => frame.len(),
        _ => 16,
    }
}

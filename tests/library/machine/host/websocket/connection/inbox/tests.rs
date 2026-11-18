// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::host::websocket::connection::inbox::Inbox;
use crate::websocket::message::Message;

fn uart(length: usize) -> Message {
    Message::Uart {
        data: "A".repeat(length),
    }
}

#[test]
fn an_inbox_gives_messages_back_in_the_order_they_came() {
    let mut inbox = Inbox::default();
    assert!(inbox.pop().is_none());
    assert!(inbox.push(uart(1)));
    assert!(inbox.push(uart(2)));
    assert_eq!(inbox.pop(), Some(uart(1)));
    assert_eq!(inbox.pop(), Some(uart(2)));
    assert!(inbox.pop().is_none());
}

#[test]
fn an_inbox_is_full_past_a_mebibyte_and_refuses_past_sixteen() {
    let mut inbox = Inbox::default();
    assert!(inbox.push(uart(1024 * 1024)));
    assert!(!inbox.is_full());
    assert!(inbox.push(uart(1)));
    assert!(inbox.is_full());
    inbox.pop();
    assert!(!inbox.is_full());
    for _ in 0..15 {
        assert!(inbox.push(uart(1024 * 1024)));
    }
    assert!(!inbox.push(uart(1024 * 1024)));
}

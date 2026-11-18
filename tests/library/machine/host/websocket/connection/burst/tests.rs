// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::host::websocket::connection::burst::Bursts;
use crate::websocket::MAX_BURST_BYTES;
use crate::websocket::base64;
use crate::websocket::message::Message;

#[test]
fn a_burst_holds_bytes_in_one_mode() {
    let mut bursts = Bursts::default();
    assert!(bursts.fits(0));
    bursts.push(0, 1);
    assert!(bursts.fits(0));
    assert!(!bursts.fits(1));
    assert_eq!(bytes(bursts.take()), [1]);
    assert!(bursts.fits(1));
}

#[test]
fn a_burst_is_full_at_4096_bytes_and_leaves_as_one_message_without_an_id() {
    let mut bursts = Bursts::default();
    for index in 1..MAX_BURST_BYTES {
        assert!(!bursts.push(3, index as u8), "{index}");
    }
    assert!(bursts.push(3, 0));
    let message = bursts.take();
    assert!(matches!(
        &message,
        Some(Message::Spi {
            selected: None,
            mode: Some(3),
            id: None,
            ..
        })
    ));
    assert_eq!(bytes(message).len(), MAX_BURST_BYTES);
    assert!(bursts.take().is_none());
}

#[test]
fn a_burst_is_settled_when_it_has_not_grown_since_it_was_last_looked_at() {
    let mut bursts = Bursts::default();
    assert!(!bursts.settled());
    bursts.push(0, 1);
    assert!(!bursts.settled());
    bursts.push(0, 2);
    assert!(!bursts.settled());
    assert!(bursts.settled());
    assert_eq!(bytes(bursts.take()), [1, 2]);
    assert!(!bursts.settled());
}

fn bytes(message: Option<Message>) -> Vec<u8> {
    match message {
        Some(Message::Spi {
            data: Some(data), ..
        }) => base64::decode(&data).unwrap(),
        other => panic!("{other:?}"),
    }
}

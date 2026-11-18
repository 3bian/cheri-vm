// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::websocket::base64::{decode, encode};

#[test]
fn bytes_of_every_length_survive_encoding_and_decoding() {
    for (bytes, text) in [
        (&b""[..], ""),
        (b"h", "aA=="),
        (b"he", "aGU="),
        (b"hel", "aGVs"),
        (b"hello\n", "aGVsbG8K"),
        (&[0x19, 0x40], "GUA="),
        (&[0xfb, 0xff, 0xfe], "+//+"),
    ] {
        assert_eq!(encode(bytes), text);
        assert_eq!(decode(text).as_deref(), Some(bytes));
    }
    let every: Vec<u8> = (0..=255).collect();
    assert_eq!(decode(&encode(&every)), Some(every));
}

#[test]
fn text_that_is_not_canonical_base64_is_refused() {
    for text in [
        "a", "aA=", "aA", "aB==", "a===", "aA==aA==", "aG-s", "aGVs\n", "=aGV",
    ] {
        assert_eq!(decode(text), None, "{text}");
    }
}

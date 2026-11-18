// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::web::handshake::{accept_key, parts};

#[test]
fn an_address_gives_where_to_connect_and_what_to_ask_for() {
    assert_eq!(
        parts("ws://127.0.0.1:9000/uart"),
        Some(("127.0.0.1:9000".into(), "/uart".into()))
    );
    assert_eq!(
        parts("ws://host/a/b?c=d"),
        Some(("host:80".into(), "/a/b?c=d".into()))
    );
    assert_eq!(parts("ws://[::1]:9"), Some(("[::1]:9".into(), "/".into())));
    for refused in [
        "ws:",
        "ws://",
        "ws:///uart",
        "ws://:9000/uart",
        "wss://host/uart",
        "http://host/uart",
        "ws://user@host/uart",
        "ws://host/a b",
        "ws://host/uart#top",
    ] {
        assert_eq!(parts(refused), None, "{refused}");
    }
}

#[test]
fn the_accept_key_is_the_one_the_standard_gives_for_its_example() {
    assert_eq!(
        accept_key("dGhlIHNhbXBsZSBub25jZQ=="),
        "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
    );
}

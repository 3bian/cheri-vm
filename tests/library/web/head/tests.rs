// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::web::head::{Head, Refused};

#[test]
fn a_head_gives_its_first_line_and_its_headers_whatever_their_case() {
    let head = Head::parse(
        b"GET /ws/uart?x=1 HTTP/1.1\r\nHost: 127.0.0.1:8080\r\n\
          UPGRADE:  websocket \r\nConnection: keep-alive, Upgrade\r\n\r\n",
    )
    .unwrap();
    assert_eq!(head.first, ["GET", "/ws/uart?x=1", "HTTP/1.1"]);
    assert_eq!(head.header("host"), Some("127.0.0.1:8080"));
    assert_eq!(head.header("Upgrade"), Some("websocket"));
    assert_eq!(head.header("origin"), None);
    assert!(head.has("connection", "upgrade"));
    assert!(head.has("upgrade", "WebSocket"));
    assert!(!head.has("connection", "close"));
    let answer =
        Head::parse(b"HTTP/1.1 101 Switching Protocols\r\n\r\n").unwrap();
    assert_eq!(answer.first[1], "101");
    assert_eq!(answer.first[2], "Switching Protocols");
}

#[test]
fn a_head_that_is_not_one_is_refused() {
    for text in [
        &b""[..],
        b"GET\r\n\r\n",
        b"GET /\r\n\r\n",
        b"GET / HTTP/1.1\r\nno colon\r\n\r\n",
        b"GET / HTTP/1.1\r\n: empty name\r\n\r\n",
        b"GET / HTTP/1.1\r\nbad name: x\r\n\r\n",
        b"GET / HTTP/1.1\r\nHost: \xff\r\n\r\n",
    ] {
        assert_eq!(Head::parse(text), Err(Refused::Bad));
    }
    let mut many = String::from("GET / HTTP/1.1\r\n");
    for index in 0..33 {
        many.push_str(&format!("X-{index}: 1\r\n"));
    }
    many.push_str("\r\n");
    assert_eq!(Head::parse(many.as_bytes()), Err(Refused::TooLarge));
}

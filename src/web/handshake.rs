// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::channel::random;
use super::head::Head;
use crate::web::digest;
use crate::websocket::base64;
use std::io::{self, Write};
use std::net::TcpStream;

const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub(crate) fn accept_key(key: &str) -> String {
    base64::encode(&digest::sha1(format!("{key}{GUID}").as_bytes()))
}

pub(crate) fn connect(
    stream: &mut TcpStream,
    authority: &str,
    path: &str,
) -> io::Result<Vec<u8>> {
    let mut nonce = Vec::with_capacity(16);
    nonce.extend_from_slice(&random().to_be_bytes());
    nonce.extend_from_slice(&random().to_be_bytes());
    let key = base64::encode(&nonce);
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {authority}\r\nUpgrade: \
         websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: \
         {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
    )?;
    stream.flush()?;
    let refused = |reason: &str| io::Error::other(reason.to_owned());
    let (head, leftover) = Head::read(stream)?
        .map_err(|_| refused("the answer is not an HTTP response"))?;
    if head.first[1] != "101" {
        return Err(refused(&format!(
            "the other end answered {} {}, not a WebSocket",
            head.first[1], head.first[2]
        )));
    }
    if !head.has("upgrade", "websocket")
        || head.header("sec-websocket-accept") != Some(&accept_key(&key))
    {
        return Err(refused("the other end did not agree to a WebSocket"));
    }
    Ok(leftover)
}

pub(crate) fn parts(url: &str) -> Option<(String, String)> {
    let rest = url.strip_prefix("ws://")?;
    let (authority, path) = match rest.find('/') {
        Some(index) => rest.split_at(index),
        None => (rest, "/"),
    };
    if authority.is_empty()
        || authority.starts_with(':')
        || authority.contains(['@', '?', '#', ' '])
        || path.contains(['#', ' '])
    {
        return None;
    }
    let has_port = authority.rsplit_once(':').is_some_and(|(_, port)| {
        !port.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit())
    });
    let authority = if has_port {
        authority.to_owned()
    } else {
        format!("{authority}:80")
    };
    Some((authority, path.to_owned()))
}

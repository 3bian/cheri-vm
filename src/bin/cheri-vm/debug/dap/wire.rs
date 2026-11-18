// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#![deny(clippy::indexing_slicing)]

use serde_json::Value;
use std::io::BufRead;

pub(super) fn base64(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut text = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let byte =
            |index: usize| u32::from(chunk.get(index).copied().unwrap_or(0));
        let bits = byte(0) << 16 | byte(1) << 8 | byte(2);
        for index in 0..4 {
            let symbol = (bits >> (18 - 6 * index) & 63) as usize;
            text.push(match ALPHABET.get(symbol) {
                Some(letter) if index <= chunk.len() => char::from(*letter),
                _ => '=',
            });
        }
    }
    text
}

pub(super) fn read_message(input: &mut impl BufRead) -> Option<Value> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if input.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length:") {
            length = value.trim().parse::<usize>().ok();
        }
    }
    let mut body = vec![0; length.filter(|length| *length <= 16 << 20)?];
    input.read_exact(&mut body).ok()?;
    serde_json::from_slice(&body).ok()
}

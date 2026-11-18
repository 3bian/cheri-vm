// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

const ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const UNUSED_BITS: [u32; 3] = [0, 0x3, 0xf];

pub fn decode(text: &str) -> Option<Vec<u8>> {
    if !is_canonical(text) {
        return None;
    }
    let symbols = text.as_bytes();
    let mut bytes = Vec::with_capacity(symbols.len() / 4 * 3);
    for chunk in symbols.chunks(4) {
        let padding = chunk
            .iter()
            .rev()
            .take_while(|symbol| **symbol == b'=')
            .count();
        let mut word = 0_u32;
        for symbol in &chunk[..4 - padding] {
            word = word << 6 | value(*symbol)?;
        }
        word <<= 6 * padding as u32;
        bytes.extend_from_slice(&word.to_be_bytes()[1..4 - padding]);
    }
    Some(bytes)
}

pub fn encode(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let word = u32::from(chunk[0]) << 16
            | u32::from(chunk.get(1).copied().unwrap_or(0)) << 8
            | u32::from(chunk.get(2).copied().unwrap_or(0));
        for position in 0..4 {
            if position <= chunk.len() {
                let index = (word >> (18 - position * 6)) & 63;
                text.push(char::from(ALPHABET[index as usize]));
            } else {
                text.push('=');
            }
        }
    }
    text
}

pub fn is_canonical(text: &str) -> bool {
    let symbols = text.as_bytes();
    if !symbols.len().is_multiple_of(4) {
        return false;
    }
    let last = symbols.len().saturating_sub(4);
    symbols.chunks(4).enumerate().all(|(index, chunk)| {
        let padding = chunk
            .iter()
            .rev()
            .take_while(|symbol| **symbol == b'=')
            .count();
        if padding > 2 || (padding != 0 && index * 4 != last) {
            return false;
        }
        let mut word = 0_u32;
        for symbol in &chunk[..4 - padding] {
            let Some(value) = value(*symbol) else {
                return false;
            };
            word = word << 6 | value;
        }
        word & UNUSED_BITS[padding] == 0
    })
}

fn value(symbol: u8) -> Option<u32> {
    let value = match symbol {
        b'A'..=b'Z' => symbol - b'A',
        b'a'..=b'z' => symbol - b'a' + 26,
        b'0'..=b'9' => symbol - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => return None,
    };
    Some(u32::from(value))
}

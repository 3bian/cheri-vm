// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{CLOSE, TEXT, mask, random};
use crate::websocket::close_reason;
use std::io::{self, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(crate) struct Writer {
    pub(super) masks: bool,
    pub(super) stream: Arc<Mutex<TcpStream>>,
}

impl Writer {
    pub(crate) fn close(&self, code: u16, reason: &str) -> io::Result<()> {
        let mut payload = code.to_be_bytes().to_vec();
        payload.extend_from_slice(close_reason(reason).as_bytes());
        self.write(CLOSE, &payload)
    }

    pub(crate) fn send(&self, text: &str) -> io::Result<()> {
        self.write(TEXT, text.as_bytes())
    }

    pub(super) fn write(&self, opcode: u8, payload: &[u8]) -> io::Result<()> {
        let mut frame = Vec::with_capacity(payload.len() + 14);
        frame.push(0x80 | opcode);
        let masked = if self.masks { 0x80 } else { 0 };
        match payload.len() {
            length @ 0..=125 => frame.push(masked | length as u8),
            length @ 126..=0xffff => {
                frame.push(masked | 126);
                frame.extend_from_slice(&(length as u16).to_be_bytes());
            }
            length => {
                frame.push(masked | 127);
                frame.extend_from_slice(&(length as u64).to_be_bytes());
            }
        }
        if self.masks {
            let key = random().to_be_bytes();
            let key = [key[0], key[1], key[2], key[3]];
            frame.extend_from_slice(&key);
            let start = frame.len();
            frame.extend_from_slice(payload);
            mask(&mut frame[start..], key);
        } else {
            frame.extend_from_slice(payload);
        }
        let mut stream = self
            .stream
            .lock()
            .map_err(|_| io::Error::other("a writer failed"))?;
        stream.write_all(&frame)?;
        stream.flush()
    }
}

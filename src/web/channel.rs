// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod event;
mod frame;
mod side;
mod writer;

pub(crate) use event::Event;
use frame::Frame;
pub(crate) use side::Side;
pub(crate) use writer::Writer;

use crate::websocket::{
    CLOSE_PROTOCOL, CLOSE_TOO_LARGE, CLOSE_UTF8, MAX_MESSAGE_BYTES,
};

use std::hash::{BuildHasher, Hasher};

use std::io::{self, Read};

use std::net::TcpStream;

use std::sync::{Arc, Mutex};

const BINARY: u8 = 0x2;
const CLOSE: u8 = 0x8;
const CONTINUATION: u8 = 0x0;
const MAX_CONTROL_BYTES: usize = 125;
const PING: u8 = 0x9;
const PONG: u8 = 0xA;
const TEXT: u8 = 0x1;

pub(crate) struct Channel {
    buffer: Vec<u8>,
    fragments: Option<(u8, Vec<u8>)>,
    side: Side,
    stream: TcpStream,
    writer: Writer,
}

impl Channel {
    pub(crate) fn close(&self, code: u16, reason: &str) -> io::Result<()> {
        self.writer.close(code, reason)
    }

    fn fail(&self, code: u16, reason: &str) -> io::Error {
        let _ = self.writer.close(code, reason);
        io::Error::new(io::ErrorKind::InvalidData, reason.to_owned())
    }

    fn frame(&mut self) -> io::Result<Option<Frame>> {
        let Some(&[first, second]) = self.buffer.first_chunk::<2>() else {
            return Ok(None);
        };
        let masked = second & 0x80 != 0;
        if first & 0x70 != 0 {
            return Err(self.fail(CLOSE_PROTOCOL, "reserved bits are set"));
        }
        if masked != (self.side == Side::Server) {
            return Err(self.fail(
                CLOSE_PROTOCOL,
                "a client masks its frames and a server does not",
            ));
        }
        let (length, mut at) = match second & 0x7f {
            126 => {
                let Some(bytes) = self.buffer.get(2..4) else {
                    return Ok(None);
                };
                (u64::from(u16::from_be_bytes([bytes[0], bytes[1]])), 4)
            }
            127 => {
                let Some(bytes) =
                    self.buffer.get(2..10).and_then(<[u8]>::first_chunk::<8>)
                else {
                    return Ok(None);
                };
                (u64::from_be_bytes(*bytes), 10)
            }
            short => (u64::from(short), 2),
        };
        let opcode = first & 0x0f;
        let limit = if opcode >= CLOSE {
            MAX_CONTROL_BYTES
        } else {
            MAX_MESSAGE_BYTES
        };
        let Some(length) = usize::try_from(length)
            .ok()
            .filter(|length| *length <= limit)
        else {
            return Err(
                self.fail(CLOSE_TOO_LARGE, "a message is at most 1 MiB")
            );
        };
        let mut key = [0_u8; 4];
        if masked {
            let Some(bytes) = self.buffer.get(at..at + 4) else {
                return Ok(None);
            };
            key.copy_from_slice(bytes);
            at += 4;
        }
        if self.buffer.len() < at + length {
            return Ok(None);
        }
        let mut payload: Vec<u8> = self.buffer.drain(..at + length).collect();
        payload.drain(..at);
        if masked {
            mask(&mut payload, key);
        }
        Ok(Some(Frame {
            last: first & 0x80 != 0,
            opcode,
            payload,
        }))
    }

    pub(crate) fn new(
        stream: TcpStream,
        side: Side,
        leftover: Vec<u8>,
    ) -> io::Result<Self> {
        let writer = Writer {
            masks: side == Side::Client,
            stream: Arc::new(Mutex::new(stream.try_clone()?)),
        };
        Ok(Self {
            buffer: leftover,
            fragments: None,
            side,
            stream,
            writer,
        })
    }

    pub(crate) fn read(&mut self) -> io::Result<Event> {
        loop {
            while let Some(frame) = self.frame()? {
                if let Some(event) = self.take(frame)? {
                    return Ok(event);
                }
            }
            let mut chunk = [0_u8; 16 * 1024];
            let count = self.stream.read(&mut chunk)?;
            if count == 0 {
                return Err(io::ErrorKind::UnexpectedEof.into());
            }
            self.buffer.extend_from_slice(&chunk[..count]);
        }
    }

    pub(crate) fn send(&self, text: &str) -> io::Result<()> {
        self.writer.send(text)
    }

    pub(crate) fn stream(&self) -> &TcpStream {
        &self.stream
    }

    fn take(&mut self, frame: Frame) -> io::Result<Option<Event>> {
        match frame.opcode {
            PING => {
                self.writer.write(PONG, &frame.payload)?;
                return Ok(None);
            }
            PONG => return Ok(None),
            CLOSE => {
                let code = frame
                    .payload
                    .first_chunk::<2>()
                    .map(|code| u16::from_be_bytes(*code));
                let reason = frame
                    .payload
                    .get(2..)
                    .map(|reason| String::from_utf8_lossy(reason).into_owned())
                    .unwrap_or_default();
                let _ = self
                    .writer
                    .write(CLOSE, frame.payload.get(..2).unwrap_or_default());
                return Ok(Some(Event::Close { code, reason }));
            }
            _ => {}
        }
        if frame.opcode > BINARY {
            return Err(self.fail(CLOSE_PROTOCOL, "unknown frame"));
        }
        let (opcode, payload) = match (self.fragments.take(), frame.opcode) {
            (Some((opcode, mut held)), CONTINUATION) => {
                if held.len() + frame.payload.len() > MAX_MESSAGE_BYTES {
                    return Err(self
                        .fail(CLOSE_TOO_LARGE, "a message is at most 1 MiB"));
                }
                held.extend_from_slice(&frame.payload);
                (opcode, held)
            }
            (None, TEXT | BINARY) => (frame.opcode, frame.payload),
            _ => {
                return Err(self.fail(CLOSE_PROTOCOL, "fragments out of order"));
            }
        };
        if !frame.last {
            self.fragments = Some((opcode, payload));
            return Ok(None);
        }
        if opcode == BINARY {
            return Ok(Some(Event::Binary));
        }
        match String::from_utf8(payload) {
            Ok(text) => Ok(Some(Event::Text(text))),
            Err(_) => Err(self.fail(CLOSE_UTF8, "text is not UTF-8")),
        }
    }

    pub(crate) fn writer(&self) -> Writer {
        self.writer.clone()
    }
}

fn mask(payload: &mut [u8], key: [u8; 4]) {
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= key[index % 4];
    }
}

pub(crate) fn random() -> u64 {
    let mut hasher =
        std::collections::hash_map::RandomState::new().build_hasher();
    hasher.write_u64(0);
    hasher.finish()
}

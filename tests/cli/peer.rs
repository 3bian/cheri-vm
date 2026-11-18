// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use tungstenite::{Message, WebSocket};

pub(crate) const BASE64: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(crate) struct Peer {
    socket: WebSocket<TcpStream>,
}

impl Peer {
    pub(crate) fn accept(listener: &TcpListener) -> Self {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .unwrap();
        Self {
            socket: tungstenite::accept(stream).unwrap(),
        }
    }

    pub(crate) fn closed_with(&mut self) -> Option<u16> {
        loop {
            match self.socket.read() {
                Ok(Message::Close(frame)) => {
                    return frame.map(|frame| frame.code.into());
                }
                Ok(_) => {}
                Err(_) => return None,
            }
        }
    }

    pub(crate) fn closed_for(&mut self) -> Option<(u16, String)> {
        loop {
            match self.socket.read() {
                Ok(Message::Close(frame)) => {
                    return frame.map(|frame| {
                        (frame.code.into(), frame.reason.as_str().to_owned())
                    });
                }
                Ok(_) => {}
                Err(_) => return None,
            }
        }
    }

    pub(crate) fn connect(url: &str) -> Self {
        let address = url
            .strip_prefix("ws://")
            .unwrap()
            .split('/')
            .next()
            .unwrap();
        let stream = TcpStream::connect(address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .unwrap();
        let (socket, _) = tungstenite::client(url, stream).unwrap();
        Self { socket }
    }

    pub(crate) fn greet(&mut self) -> Value {
        let hello = self.receive().unwrap();
        assert_eq!(hello["type"], "hello", "{hello}");
        assert_eq!(hello["format_version"], 1, "{hello}");
        hello
    }

    pub(crate) fn receive(&mut self) -> Option<Value> {
        loop {
            let message = self.receive_or_sync()?;
            if message["type"] != "sync" {
                return Some(message);
            }
            self.send(&json!({
                "type": "reply",
                "id": message["id"],
                "data": "",
            }));
        }
    }

    pub(crate) fn receive_or_sync(&mut self) -> Option<Value> {
        loop {
            match self.socket.read() {
                Ok(Message::Text(text)) => {
                    return Some(serde_json::from_str(&text).unwrap());
                }
                Ok(Message::Close(_)) | Err(_) => return None,
                Ok(_) => {}
            }
        }
    }

    pub(crate) fn send(&mut self, value: &Value) {
        self.socket.send(Message::text(value.to_string())).unwrap();
    }

    pub(crate) fn send_binary(&mut self, bytes: &[u8]) {
        self.socket.send(Message::binary(bytes.to_vec())).unwrap();
    }

    pub(crate) fn until_bye(&mut self) -> (Vec<Value>, Value) {
        let mut seen = Vec::new();
        loop {
            let message = self
                .receive()
                .unwrap_or_else(|| panic!("closed after {seen:?}"));
            if message["type"] == "bye" {
                return (seen, message);
            }
            seen.push(message);
        }
    }
}

pub(crate) fn decode(text: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    for chunk in text.as_bytes().chunks(4) {
        let symbols: Vec<u32> = chunk
            .iter()
            .filter(|symbol| **symbol != b'=')
            .map(|symbol| {
                BASE64.iter().position(|other| other == symbol).unwrap() as u32
            })
            .collect();
        let word = symbols.iter().fold(0, |word, symbol| word << 6 | symbol)
            << (6 * (4 - symbols.len()));
        bytes.extend_from_slice(&word.to_be_bytes()[1..symbols.len()]);
    }
    bytes
}

pub(crate) fn encode(bytes: &[u8]) -> String {
    let mut text = String::new();
    for chunk in bytes.chunks(3) {
        let word = u32::from(chunk[0]) << 16
            | u32::from(*chunk.get(1).unwrap_or(&0)) << 8
            | u32::from(*chunk.get(2).unwrap_or(&0));
        for position in 0..4 {
            if position <= chunk.len() {
                text.push(char::from(
                    BASE64[(word >> (18 - position * 6) & 63) as usize],
                ));
            } else {
                text.push('=');
            }
        }
    }
    text
}

pub(crate) fn listen() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("ws://{}/desk", listener.local_addr().unwrap());
    (listener, url)
}

pub(crate) fn play(peers: Vec<Peer>) -> Vec<Value> {
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let players: Vec<_> = peers
        .into_iter()
        .map(|mut peer| {
            let seen = seen.clone();
            std::thread::spawn(move || {
                while let Some(message) = peer.receive_or_sync() {
                    if message["type"] == "bye" {
                        break;
                    }
                    let answer = if message["type"] == "sync" {
                        ""
                    } else {
                        seen.lock().unwrap().push(message.clone());
                        "/w=="
                    };
                    if let Some(id) = message.get("id") {
                        peer.send(&json!({
                            "type": "reply",
                            "id": id,
                            "data": answer,
                        }));
                    }
                }
            })
        })
        .collect();
    for player in players {
        player.join().unwrap();
    }
    std::sync::Arc::try_unwrap(seen)
        .unwrap()
        .into_inner()
        .unwrap()
}

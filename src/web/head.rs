// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod refused;

pub(crate) use refused::Refused;

use std::io::{self, Read};

use std::net::TcpStream;

use std::time::{Duration, Instant};

const HEAD_TIMEOUT: Duration = Duration::from_secs(10);
pub(crate) const MAX_HEAD: usize = 8 * 1024;
const MAX_HEADERS: usize = 32;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Head {
    pub first: [String; 3],
    pub headers: Vec<(String, String)>,
}

impl Head {
    pub(crate) fn has(&self, name: &str, token: &str) -> bool {
        self.header(name).is_some_and(|value| {
            value
                .split(',')
                .any(|part| part.trim().eq_ignore_ascii_case(token))
        })
    }

    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(header, _)| header.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    pub(crate) fn parse(text: &[u8]) -> Result<Self, Refused> {
        let text = std::str::from_utf8(text).map_err(|_| Refused::Bad)?;
        let mut lines = text.split("\r\n");
        let mut words = lines.next().unwrap_or_default().splitn(3, ' ');
        let mut word = || {
            words
                .next()
                .filter(|word| !word.is_empty())
                .map(str::to_owned)
                .ok_or(Refused::Bad)
        };
        let first = [word()?, word()?, word()?];
        let mut headers = Vec::new();
        for line in lines.take_while(|line| !line.is_empty()) {
            let (name, value) = line.split_once(':').ok_or(Refused::Bad)?;
            let named = !name.is_empty()
                && name.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || b"-_.".contains(&byte)
                });
            if !named {
                return Err(Refused::Bad);
            }
            if headers.len() == MAX_HEADERS {
                return Err(Refused::TooLarge);
            }
            headers.push((name.to_owned(), value.trim().to_owned()));
        }
        Ok(Self { first, headers })
    }

    pub(crate) fn read(
        stream: &mut TcpStream,
    ) -> io::Result<Result<(Self, Vec<u8>), Refused>> {
        let mut head = Vec::with_capacity(1024);
        let mut chunk = [0_u8; 1024];
        let started = Instant::now();
        stream.set_read_timeout(Some(HEAD_TIMEOUT))?;
        loop {
            if started.elapsed() > HEAD_TIMEOUT {
                return Err(io::ErrorKind::TimedOut.into());
            }
            let count = stream.read(&mut chunk)?;
            if count == 0 {
                return Err(io::ErrorKind::UnexpectedEof.into());
            }
            let from = head.len().saturating_sub(3);
            head.extend_from_slice(&chunk[..count]);
            if let Some(index) = head[from..]
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
            {
                let rest = head.split_off(from + index + 4);
                stream.set_read_timeout(None)?;
                if head.len() > MAX_HEAD {
                    return Ok(Err(Refused::TooLarge));
                }
                return Ok(Self::parse(&head).map(|head| (head, rest)));
            }
            if head.len() > MAX_HEAD {
                return Ok(Err(Refused::TooLarge));
            }
        }
    }
}

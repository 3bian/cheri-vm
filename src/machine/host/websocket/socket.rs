// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::EmulatorError;
use crate::web::channel::{Channel, Event, Side};
use crate::websocket::CLOSE_BINARY;
use std::io;
use std::net::TcpStream;
use std::time::{Duration, Instant};

pub(super) const FOREVER: Option<Duration> = None;
pub(super) const POLL: Option<Duration> = Some(Duration::ZERO);

pub(super) struct Socket {
    channel: Channel,
}

impl Socket {
    const CLOSE_TIMEOUT: Duration = Duration::from_millis(500);
    const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

    pub(super) fn close(&mut self, code: u16, reason: &str) {
        let _ = self.channel.stream().set_nonblocking(false);
        let _ = self.channel.close(code, reason);
        let deadline = Instant::now() + Self::CLOSE_TIMEOUT;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero()
                || self.channel.stream().set_read_timeout(Some(left)).is_err()
                || !matches!(self.channel.read(), Ok(Event::Text(_)))
            {
                break;
            }
        }
    }

    pub(super) fn open(url: &str) -> Result<Self, EmulatorError> {
        use std::net::ToSocketAddrs;
        let refused = |error: &dyn std::fmt::Display| {
            EmulatorError::HostIo(
                io::Error::other(format!(
                    "could not connect to {url}: {error}"
                ))
                .into(),
            )
        };
        let (authority, path) = crate::web::handshake::parts(url)
            .ok_or_else(|| refused(&"it is not a ws:// address"))?;
        let mut failure = None;
        let stream = authority
            .to_socket_addrs()
            .map_err(|error| refused(&error))?
            .find_map(|address| {
                TcpStream::connect_timeout(&address, Self::CONNECT_TIMEOUT)
                    .map_err(|error| failure = Some(error))
                    .ok()
            });
        let Some(mut stream) = stream else {
            return Err(match failure {
                Some(error) => refused(&error),
                None => refused(&"the name has no address"),
            });
        };
        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(Self::CONNECT_TIMEOUT))?;
        let leftover =
            crate::web::handshake::connect(&mut stream, &authority, &path)
                .map_err(|error| refused(&error))?;
        stream.set_read_timeout(None)?;
        Ok(Self {
            channel: Channel::new(stream, Side::Client, leftover)?,
        })
    }

    fn read(&mut self) -> io::Result<Option<String>> {
        match self.channel.read() {
            Ok(Event::Text(text)) => Ok(Some(text)),
            Ok(Event::Binary) => {
                self.close(CLOSE_BINARY, "text frames only");
                Err(io::Error::other("the other end sent a binary frame"))
            }
            Ok(Event::Close { reason, .. }) => {
                Err(io::Error::other(if reason.is_empty() {
                    "closed".to_owned()
                } else {
                    format!("closed: {reason}")
                }))
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                Ok(None)
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::BrokenPipe
                        | io::ErrorKind::ConnectionAborted
                        | io::ErrorKind::ConnectionReset
                        | io::ErrorKind::UnexpectedEof
                ) =>
            {
                Err(io::Error::other("closed"))
            }
            Err(error) => Err(io::Error::other(error.to_string())),
        }
    }

    pub(super) fn receive(
        &mut self,
        wait: Option<Duration>,
    ) -> io::Result<Option<String>> {
        let stream = self.channel.stream();
        if wait == POLL {
            stream.set_nonblocking(true)?;
        } else {
            stream.set_read_timeout(wait)?;
        }
        let received = self.read();
        if wait == POLL {
            self.channel.stream().set_nonblocking(false)?;
        }
        received
    }

    pub(super) fn send(&mut self, text: &str) -> io::Result<()> {
        self.channel.send(text).map_err(|error| {
            if matches!(
                error.kind(),
                io::ErrorKind::BrokenPipe
                    | io::ErrorKind::ConnectionAborted
                    | io::ErrorKind::ConnectionReset
            ) {
                io::Error::other("closed")
            } else {
                io::Error::other(error.to_string())
            }
        })
    }

    pub(super) fn served(stream: TcpStream) -> io::Result<Self> {
        Ok(Self {
            channel: Channel::new(stream, Side::Client, Vec::new())?,
        })
    }
}

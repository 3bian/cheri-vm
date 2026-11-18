// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::cell::Cell;
use std::io::{self, Write};
use std::rc::Rc;
use std::time::{Duration, Instant};

use super::socket::{FOREVER, POLL, Socket};
use crate::machine::EmulatorError;
use crate::websocket::base64;
use crate::websocket::endpoint::Kind;
use crate::websocket::message::{Message, Reason, ReplyError, Role};
use crate::websocket::{CLOSE_NORMAL, CLOSE_RULE, FORMAT_VERSION};
use burst::Bursts;
use inbox::Inbox;

pub(crate) mod burst;
pub(crate) mod inbox;

pub(super) struct Connection {
    awaited: Option<u64>,
    bursts: Bursts,
    pub(super) device: String,
    failure: Option<String>,
    inbox: Inbox,
    kind: Option<Kind>,
    last_pump: Instant,
    next_id: u64,
    reply: Option<Result<Vec<u8>, String>>,
    served: bool,
    socket: Socket,
    unsynced: bool,
    url: String,
    waited: Rc<Cell<Duration>>,
}

impl Connection {
    const PUMP_PERIOD: Duration = Duration::from_micros(250);
    const QUIET_BEFORE_NOTICE: Duration = Duration::from_millis(250);

    fn accept(&mut self, text: &str) -> io::Result<()> {
        let message = match Message::parse(text, Role::Device) {
            Ok(message) => message,
            Err(error) => return Err(self.break_rule(&error)),
        };
        match message {
            Message::Reply {
                id,
                data,
                error,
                message,
            } if self.awaited == Some(id) => {
                self.reply = Some(match (data, error) {
                    (Some(data), _) => {
                        Ok(base64::decode(&data).unwrap_or_default())
                    }
                    (None, error) => Err(format!(
                        "the other end answered {}{}",
                        if error == Some(ReplyError::Nack) {
                            "nack"
                        } else {
                            "failed"
                        },
                        message
                            .map(|text| format!(": {text}"))
                            .unwrap_or_default()
                    )),
                });
                Ok(())
            }
            Message::Reply { id, .. } => Err(self.break_rule(&format!(
                "no message with id {id} awaits a reply"
            ))),
            Message::Error { message } => {
                Err(self.fail(&format!("refused: {message}")))
            }
            Message::Uart { .. }
            | Message::Gpio { .. }
            | Message::Ethernet { .. } => {
                let fits = matches!(
                    (&message, self.kind),
                    (Message::Uart { .. }, Some(Kind::Uart))
                        | (Message::Gpio { .. }, Some(Kind::Gpio))
                        | (Message::Ethernet { .. }, Some(Kind::Ethernet))
                );
                if !fits {
                    Err(self.break_rule(&format!(
                        "{} is not a message for {}",
                        message.name(),
                        self.device
                    )))
                } else if self.inbox.push(message) {
                    Ok(())
                } else {
                    Err(self.break_rule("too much input is waiting"))
                }
            }
            other => Err(self
                .break_rule(&format!("{} is not expected here", other.name()))),
        }
    }

    pub(super) fn announce(
        &mut self,
        profile: &str,
        kind: Kind,
        write_only: bool,
    ) -> Result<(), EmulatorError> {
        self.kind = Some(kind);
        self.send(&Message::Hello {
            format_version: FORMAT_VERSION,
            profile: profile.to_owned(),
            device: self.device.clone(),
            kind,
            write_only,
        })?;
        Ok(())
    }

    pub(super) fn await_ready(&mut self) -> Result<(), EmulatorError> {
        let notice = format!("{:>12} {}", "Waiting", self.url);
        let mut wait = if self.served {
            writeln!(io::stderr().lock(), "{notice}")?;
            FOREVER
        } else {
            Some(Self::QUIET_BEFORE_NOTICE)
        };
        loop {
            let Some(text) = self.receive(wait)? else {
                writeln!(io::stderr().lock(), "{notice}")?;
                wait = FOREVER;
                continue;
            };
            match Message::parse(&text, Role::Device) {
                Ok(Message::Ready {}) => return Ok(()),
                Ok(Message::Error { message }) => {
                    return Err(self
                        .fail(&format!("refused: {message}"))
                        .into());
                }
                Ok(message) => {
                    return Err(self
                        .break_rule(&format!(
                            "{} came before ready",
                            message.name()
                        ))
                        .into());
                }
                Err(error) => return Err(self.break_rule(&error).into()),
            }
        }
    }

    fn break_rule(&mut self, rule: &str) -> io::Error {
        let _ = self.socket.send(
            &Message::Error {
                message: rule.to_owned(),
            }
            .text(),
        );
        self.socket.close(CLOSE_RULE, rule);
        self.fail(&format!("broke a rule: {rule}"))
    }

    pub(super) fn buffer(&mut self, mode: u8, byte: u8) -> io::Result<()> {
        if !self.bursts.fits(mode) {
            self.flush()?;
        }
        if self.bursts.push(mode, byte) {
            self.flush()?;
        }
        Ok(())
    }

    fn check(&self) -> io::Result<()> {
        match &self.failure {
            Some(failure) => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                failure.clone(),
            )),
            None => Ok(()),
        }
    }

    fn fail(&mut self, cause: &str) -> io::Error {
        let failure =
            format!("{} {cause}; {} has no host", self.url, self.device);
        self.failure = Some(failure.clone());
        io::Error::new(io::ErrorKind::NotConnected, failure)
    }

    pub(super) fn finish(&mut self, reason: Reason, message: &str) {
        if self.failure.is_some() {
            return;
        }
        let sent = self.send(&Message::Bye {
            reason,
            message: message.to_owned(),
        });
        if sent.is_ok() {
            self.socket.close(CLOSE_NORMAL, "");
            self.failure = Some(format!("{} is finished", self.url));
        }
    }

    pub(super) fn flush(&mut self) -> io::Result<()> {
        match self.bursts.take() {
            Some(message) => self.transmit(&message),
            None => Ok(()),
        }
    }

    fn new(
        socket: Socket,
        url: &str,
        device: &str,
        served: bool,
        waited: Rc<Cell<Duration>>,
    ) -> Self {
        Self {
            awaited: None,
            bursts: Bursts::default(),
            device: device.to_owned(),
            failure: None,
            inbox: Inbox::default(),
            kind: None,
            last_pump: Instant::now(),
            next_id: 1,
            reply: None,
            served,
            socket,
            unsynced: false,
            url: url.to_owned(),
            waited,
        }
    }

    pub(super) fn open(
        url: &str,
        device: &str,
        waited: Rc<Cell<Duration>>,
    ) -> Result<Self, EmulatorError> {
        Ok(Self::new(Socket::open(url)?, url, device, false, waited))
    }

    fn pump(&mut self) -> io::Result<()> {
        self.check()?;
        if self.last_pump.elapsed() < Self::PUMP_PERIOD {
            return Ok(());
        }
        while !self.inbox.is_full() {
            let Some(text) = self.receive(POLL)? else {
                break;
            };
            self.accept(&text)?;
        }
        self.last_pump = Instant::now();
        Ok(())
    }

    fn receive(
        &mut self,
        wait: Option<Duration>,
    ) -> io::Result<Option<String>> {
        self.check()?;
        self.socket
            .receive(wait)
            .map_err(|error| self.fail(&error.to_string()))
    }

    pub(super) fn request(
        &mut self,
        timeout: Duration,
        build: impl FnOnce(u64) -> Message,
    ) -> io::Result<Vec<u8>> {
        let id = self.next_id;
        self.next_id += 1;
        let message = build(id);
        self.send(&message)?;
        self.awaited = Some(id);
        let started = Instant::now();
        let outcome = loop {
            if let Some(reply) = self.reply.take() {
                break reply.map_err(io::Error::other);
            }
            let Some(remaining) = timeout
                .checked_sub(started.elapsed())
                .filter(|remaining| !remaining.is_zero())
            else {
                break Err(self.fail(&format!(
                    "no reply to {} within {} ms",
                    message.name(),
                    timeout.as_millis()
                )));
            };
            match self.receive(Some(remaining)) {
                Ok(Some(text)) => {
                    if let Err(error) = self.accept(&text) {
                        break Err(error);
                    }
                }
                Ok(None) => {}
                Err(error) => break Err(error),
            }
        };
        self.awaited = None;
        self.unsynced = outcome.is_err();
        self.waited.set(self.waited.get() + started.elapsed());
        outcome
    }

    pub(super) fn send(&mut self, message: &Message) -> io::Result<()> {
        self.flush()?;
        self.transmit(message)
    }

    pub(super) fn served(
        stream: std::net::TcpStream,
        url: &str,
        device: &str,
        waited: Rc<Cell<Duration>>,
    ) -> Result<Self, EmulatorError> {
        Ok(Self::new(
            Socket::served(stream)?,
            url,
            device,
            true,
            waited,
        ))
    }

    pub(super) fn settle(&mut self) -> io::Result<()> {
        if self.bursts.settled() {
            self.flush()?;
        }
        Ok(())
    }

    pub(super) fn sync(&mut self, timeout: Duration) -> io::Result<()> {
        if self.failure.is_some() {
            return Ok(());
        }
        self.flush()?;
        if self.unsynced {
            self.request(timeout, |id| Message::Sync { id })?;
        }
        Ok(())
    }

    pub(super) fn take(&mut self) -> io::Result<Option<Message>> {
        self.pump()?;
        Ok(self.inbox.pop())
    }

    fn transmit(&mut self, message: &Message) -> io::Result<()> {
        self.check()?;
        self.unsynced = true;
        self.socket
            .send(&message.text())
            .map_err(|error| self.fail(&error.to_string()))
    }
}

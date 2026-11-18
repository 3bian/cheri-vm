// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::channel::{Channel, Event, Side};
use super::{Slot, WRITE_TIMEOUT, finish};
use crate::websocket::{CLOSE_AWAY, CLOSE_BINARY, CLOSE_NORMAL, CLOSE_RULE};
use std::io::{self};
use std::net::Shutdown;
use std::sync::Mutex;

pub(super) struct Device {
    pub(super) name: String,
    pub(super) slot: Mutex<Slot>,
}

impl Device {
    pub(super) fn serve(&self, mut page: Channel) -> io::Result<()> {
        let taken = match self.slot.lock() {
            Ok(mut slot) => match std::mem::replace(&mut *slot, Slot::Taken) {
                Slot::Free(end) => Ok(end),
                other => {
                    let gone = matches!(other, Slot::Gone);
                    *slot = other;
                    Err(gone)
                }
            },
            Err(_) => Err(true),
        };
        let end = match taken {
            Ok(end) => end,
            Err(gone) => {
                let reason = if gone {
                    format!("{} has gone: its page closed", self.name)
                } else {
                    format!("{} already has a page", self.name)
                };
                page.close(CLOSE_RULE, &reason)?;
                finish(page.stream().try_clone()?);
                return Ok(());
            }
        };
        end.set_write_timeout(Some(WRITE_TIMEOUT))?;
        let closer = end.try_clone()?;
        let mut machine = Channel::new(end, Side::Server, Vec::new())?;
        let to_machine = machine.writer();
        let to_page = page.writer();
        let leaving = page.stream().try_clone()?;
        let outward = std::thread::Builder::new()
            .name(format!("web {}", self.name))
            .spawn(move || {
                loop {
                    match machine.read() {
                        Ok(Event::Text(text)) => {
                            if to_page.send(&text).is_err() {
                                break;
                            }
                        }
                        Ok(Event::Binary) => {}
                        Ok(Event::Close { code, reason }) => {
                            let code = code.unwrap_or(CLOSE_NORMAL);
                            let _ = to_page.close(code, &reason);
                            break;
                        }
                        Err(_) => {
                            let _ = to_page.close(CLOSE_AWAY, "the run ended");
                            break;
                        }
                    }
                }
                let _ = leaving.shutdown(Shutdown::Write);
            })?;
        loop {
            match page.read() {
                Ok(Event::Text(text)) => {
                    if to_machine.send(&text).is_err() {
                        break;
                    }
                }
                Ok(Event::Binary) => {
                    let _ = page.close(CLOSE_BINARY, "text frames only");
                    let reason = "the page sent a binary frame";
                    let _ = to_machine.close(CLOSE_BINARY, reason);
                    break;
                }
                Ok(Event::Close { .. }) | Err(_) => {
                    let _ = to_machine.close(CLOSE_AWAY, "the page closed");
                    break;
                }
            }
        }
        let _ = closer.shutdown(Shutdown::Write);
        if let Ok(mut slot) = self.slot.lock() {
            *slot = Slot::Gone;
        }
        let _ = outward.join();
        Ok(())
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::time::Duration;

use super::connection::Connection;
use crate::websocket::message::Message;

#[derive(Clone)]
pub(crate) struct Link {
    pub(super) connection: Rc<RefCell<Connection>>,
    pub(super) last: Rc<RefCell<Option<Rc<RefCell<Connection>>>>>,
    pub(super) timeout: Duration,
}

impl Link {
    pub(super) fn buffer(&self, mode: u8, byte: u8) -> io::Result<()> {
        self.turn()?;
        self.connection.borrow_mut().buffer(mode, byte)
    }

    pub(super) fn device(&self) -> String {
        self.connection.borrow().device.clone()
    }

    pub(super) fn request(
        &self,
        build: impl FnOnce(u64) -> Message,
    ) -> io::Result<Vec<u8>> {
        self.turn()?;
        self.connection.borrow_mut().request(self.timeout, build)
    }

    pub(super) fn send(&self, message: &Message) -> io::Result<()> {
        self.turn()?;
        self.connection.borrow_mut().send(message)
    }

    pub(super) fn take(&self) -> io::Result<Option<Message>> {
        self.connection.borrow_mut().take()
    }

    fn turn(&self) -> io::Result<()> {
        let last = self.last.replace(Some(self.connection.clone()));
        match last {
            Some(last) if !Rc::ptr_eq(&last, &self.connection) => {
                last.borrow_mut().sync(self.timeout)
            }
            _ => Ok(()),
        }
    }
}

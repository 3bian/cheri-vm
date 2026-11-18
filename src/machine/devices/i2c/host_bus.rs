// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::I2cTransport;
use super::registers::{
    COMMAND_READ, COMMAND_RESTART, COMMAND_STOP, COMMAND_WRITE,
    STATUS_ACKNOWLEDGED, STATUS_ACTIVE, STATUS_COMPLETE,
};
use crate::machine::error::{HostError, HostErrorKind, HostResult};

const BUSY_AND_ACKNOWLEDGED: u32 =
    STATUS_COMPLETE | STATUS_ACTIVE | STATUS_ACKNOWLEDGED;

pub(super) struct HostBus {
    address: Option<u8>,
    pub(super) addresses: Option<Vec<u8>>,
    next_read: usize,
    read: Vec<u8>,
    reading: bool,
    restart: bool,
    shared: bool,
    transport: Box<dyn I2cTransport>,
    write: Vec<u8>,
}

impl HostBus {
    pub(crate) fn command(
        &mut self,
        command: u32,
        address: u8,
        data: u8,
        read_length: usize,
    ) -> HostResult<(u8, u32)> {
        if command == COMMAND_STOP {
            if !self.write.is_empty() {
                self.transport.transfer(
                    self.address
                        .ok_or_else(|| invalid("missing I2C address"))?,
                    &self.write,
                    &mut [],
                )?;
            }
            self.address = None;
            self.next_read = 0;
            self.read.clear();
            self.reading = false;
            self.restart = false;
            self.write.clear();
            return Ok((data, STATUS_COMPLETE | STATUS_ACKNOWLEDGED));
        }
        if !(0x08..=0x77).contains(&address)
            || self
                .addresses
                .as_ref()
                .is_some_and(|addresses| !addresses.contains(&address))
        {
            return Err(HostError::new(
                HostErrorKind::PermissionDenied,
                format!("host I2C address 0x{address:02x} is not permitted"),
            ));
        }
        if self.address.is_some_and(|previous| previous != address) {
            return Err(invalid(
                "host I2C requires STOP before changing address",
            ));
        }
        match command {
            COMMAND_WRITE => {
                if self.reading || self.restart {
                    return Err(invalid(
                        "host I2C requires STOP before writing after a read",
                    ));
                }
                if self.write.len() == 4096 {
                    return Err(invalid("host I2C write exceeds 4096 bytes"));
                }
                self.write.push(data);
                self.address = Some(address);
                Ok((data, BUSY_AND_ACKNOWLEDGED))
            }
            COMMAND_READ => {
                if !self.reading {
                    self.read.resize(read_length, 0);
                    if let Err(error) = self.transport.transfer(
                        address,
                        &self.write,
                        &mut self.read,
                    ) {
                        self.read.clear();
                        return Err(error);
                    }
                    self.write.clear();
                    self.address = Some(address);
                    self.reading = true;
                }
                let value = self.read.get(self.next_read).copied().ok_or_else(
                    || {
                        invalid(
                            "host I2C read burst exhausted; set READ_LENGTH \
                             before starting and issue STOP between bursts",
                        )
                    },
                )?;
                self.next_read += 1;
                Ok((value, BUSY_AND_ACKNOWLEDGED))
            }
            COMMAND_RESTART if !self.reading => {
                self.address = Some(address);
                self.restart = true;
                Ok((data, BUSY_AND_ACKNOWLEDGED))
            }
            _ => Err(invalid(
                "host I2C supports writes, write-then-read repeated START and \
                 STOP; other sequences require STOP first",
            )),
        }
    }

    pub(crate) fn new(transport: Box<dyn I2cTransport>) -> Self {
        Self {
            address: None,
            addresses: None,
            next_read: 0,
            read: Vec::new(),
            reading: false,
            restart: false,
            shared: false,
            transport,
            write: Vec::new(),
        }
    }

    pub(crate) fn serves(&self, command: u32, address: u8) -> bool {
        if !self.shared {
            return true;
        }
        if command == COMMAND_STOP {
            return self.address.is_some();
        }
        self.addresses
            .as_ref()
            .is_some_and(|addresses| addresses.contains(&address))
    }

    pub(crate) fn shared(
        transport: Box<dyn I2cTransport>,
        addresses: Vec<u8>,
    ) -> Self {
        Self {
            addresses: Some(addresses),
            shared: true,
            ..Self::new(transport)
        }
    }
}

fn invalid(message: &str) -> HostError {
    HostError::new(HostErrorKind::InvalidInput, message)
}

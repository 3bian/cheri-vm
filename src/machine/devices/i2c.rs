// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::error::HostError;
use std::collections::BTreeMap;

use crate::machine::devices::{Device, DeviceActivity, InterruptLine};
use crate::machine::error::EmulatorError;

mod address_filter;
mod eeprom;
mod host_bus;
mod i2c_peripheral;
pub(crate) mod registers;
mod transport;

pub(crate) use address_filter::validate_host_addresses;
pub(crate) use eeprom::Eeprom;
pub(crate) use i2c_peripheral::I2cPeripheral;
use registers::{
    ADDRESS_MAX, COMMAND_READ, COMMAND_RESTART, COMMAND_STOP, COMMAND_WRITE,
    CONTROL_INTERRUPT_ENABLE, CONTROL_READ_LENGTH, CONTROL_READ_LENGTH_SHIFT,
    Register, STATUS_ACKNOWLEDGED, STATUS_ACTIVE, STATUS_COMPLETE,
    STATUS_ERROR,
};
pub(crate) use transport::I2cTransport;

pub(crate) struct I2c {
    active: bool,
    active_address: Option<u8>,
    address: u8,
    control: u32,
    data: u8,
    host: Option<host_bus::HostBus>,
    host_error: Option<String>,
    peripherals: BTreeMap<u8, Box<dyn I2cPeripheral>>,
    reading: bool,
    restart: bool,
    status: u32,
}

impl I2c {
    pub(crate) fn attach(
        &mut self,
        address: u8,
        peripheral: Box<dyn I2cPeripheral>,
    ) -> Result<(), EmulatorError> {
        if u32::from(address) > ADDRESS_MAX
            || self.peripherals.contains_key(&address)
        {
            return Err(EmulatorError::Configuration(
                "I2C address must be a unique 7-bit address".into(),
            ));
        }

        self.peripherals.insert(address, peripheral);

        Ok(())
    }

    pub(crate) fn new() -> Self {
        Self {
            peripherals: BTreeMap::new(),
            address: 0,
            data: 0,
            host: None,
            host_error: None,
            status: 0,
            control: 0,
            active: false,
            active_address: None,
            reading: false,
            restart: false,
        }
    }

    pub(crate) fn set_host_addresses(
        &mut self,
        addresses: Vec<u8>,
    ) -> Result<(), EmulatorError> {
        validate_host_addresses(&addresses)?;
        if self.active {
            return Err(EmulatorError::Configuration(
                "I2C address policy cannot change during a transaction".into(),
            ));
        }
        let host = self.host.as_mut().ok_or_else(|| {
            EmulatorError::Configuration(
                "I2C host address policy requires a host transport".into(),
            )
        })?;
        host.addresses = Some(addresses);
        Ok(())
    }

    pub(crate) fn share_with_host(
        &mut self,
        transport: Box<dyn I2cTransport>,
        addresses: Vec<u8>,
    ) -> Result<(), EmulatorError> {
        validate_host_addresses(&addresses)?;
        if self.active || self.host.is_some() {
            return Err(EmulatorError::Configuration(
                "I2C host cannot change during a transaction or be bound twice"
                    .into(),
            ));
        }
        self.host = Some(host_bus::HostBus::shared(transport, addresses));
        Ok(())
    }

    pub(crate) fn with_host(transport: Box<dyn I2cTransport>) -> Self {
        Self {
            host: Some(host_bus::HostBus::new(transport)),
            ..Self::new()
        }
    }
}

impl Default for I2c {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for I2c {
    fn activity(&self) -> DeviceActivity {
        DeviceActivity {
            host_errors: self.host.is_some(),
            interrupt_lines: &[InterruptLine::MachineExternal],
            ..DeviceActivity::default()
        }
    }

    fn host_error(&self) -> Option<&str> {
        self.host_error.as_deref()
    }

    fn interrupt(&self) -> Option<InterruptLine> {
        (self.control & CONTROL_INTERRUPT_ENABLE != 0
            && self.status & STATUS_COMPLETE != 0)
            .then_some(InterruptLine::MachineExternal)
    }

    fn read(
        &mut self,
        offset: u32,
        width: usize,
    ) -> Result<u32, EmulatorError> {
        Ok(match Register::access(offset, width)? {
            Register::Address => u32::from(self.address),
            Register::Data => u32::from(self.data),
            Register::Command | Register::Acknowledge => 0,
            Register::Status => self.status,
            Register::Control => self.control,
        })
    }

    fn write(
        &mut self,
        offset: u32,
        width: usize,
        value: u32,
    ) -> Result<(), EmulatorError> {
        let register = Register::access(offset, width)?;

        if let Some(error) = &self.host_error {
            return Err(EmulatorError::HostIo(HostError::other(error.clone())));
        }
        match register {
            Register::Address => {
                if value > ADDRESS_MAX {
                    return Err(EmulatorError::Bus {
                        address: u64::from(offset),
                        explanation: "I2C address exceeds seven bits".into(),
                    });
                }

                self.address = value as u8;
            }
            Register::Data => self.data = value as u8,
            Register::Command => {
                if !(COMMAND_WRITE..=COMMAND_RESTART).contains(&value) {
                    return Err(EmulatorError::Bus {
                        address: u64::from(offset),
                        explanation: "invalid I2C command".into(),
                    });
                }

                if let Some(host) = &mut self.host
                    && host.serves(value, self.address)
                {
                    match host.command(
                        value,
                        self.address,
                        self.data,
                        (((self.control & CONTROL_READ_LENGTH)
                            >> CONTROL_READ_LENGTH_SHIFT)
                            + 1) as usize,
                    ) {
                        Ok((data, status)) => {
                            self.data = data;
                            self.status = status;
                            self.active = status & STATUS_ACTIVE != 0;
                            return Ok(());
                        }
                        Err(error) => {
                            let message =
                                format!("I2C transport failed: {error}");
                            self.host_error = Some(message.clone());
                            return Err(EmulatorError::HostIo(
                                HostError::other(message),
                            ));
                        }
                    }
                }
                self.status = STATUS_COMPLETE;

                if value == COMMAND_WRITE || value == COMMAND_READ {
                    let reading = value == COMMAND_READ;

                    if self.restart
                        || self.active_address != Some(self.address)
                        || self.reading != reading
                    {
                        let acknowledged = self
                            .peripherals
                            .get_mut(&self.address)
                            .is_some_and(|peripheral| {
                                peripheral.start(reading)
                            });
                        self.active_address = Some(self.address);
                        self.active = true;
                        self.reading = reading;
                        self.restart = false;

                        if !acknowledged {
                            self.active_address = None;
                            self.status |= STATUS_ERROR | STATUS_ACTIVE;
                            return Ok(());
                        }
                    }
                }

                match value {
                    COMMAND_WRITE => {
                        self.active = true;
                        let acknowledged = self
                            .peripherals
                            .get_mut(&self.address)
                            .is_some_and(|peripheral| {
                                peripheral.write_byte(self.data)
                            });
                        self.status |= if acknowledged {
                            STATUS_ACKNOWLEDGED
                        } else {
                            STATUS_ERROR
                        };
                    }
                    COMMAND_READ => {
                        self.active = true;

                        if let Some(peripheral) =
                            self.peripherals.get_mut(&self.address)
                        {
                            self.data = peripheral.read_byte();
                            self.status |= STATUS_ACKNOWLEDGED;
                        } else {
                            self.status |= STATUS_ERROR;
                        }
                    }
                    COMMAND_STOP => {
                        if let Some(address) = self.active_address.take()
                            && let Some(peripheral) =
                                self.peripherals.get_mut(&address)
                        {
                            peripheral.stop();
                        }

                        self.restart = false;
                        self.active = false;
                        self.status |= STATUS_ACKNOWLEDGED;
                    }
                    COMMAND_RESTART => {
                        self.active = true;
                        self.restart = true;
                        self.status |=
                            if self.peripherals.contains_key(&self.address) {
                                STATUS_ACKNOWLEDGED
                            } else {
                                STATUS_ERROR
                            };
                    }
                    _ => {
                        return Err(EmulatorError::Bus {
                            address: u64::from(offset),
                            explanation: "invalid I2C command".into(),
                        });
                    }
                }

                if self.active {
                    self.status |= STATUS_ACTIVE;
                }
            }
            Register::Control => {
                self.control =
                    value & (CONTROL_INTERRUPT_ENABLE | CONTROL_READ_LENGTH);
            }
            Register::Acknowledge => self.status &= !(value & STATUS_COMPLETE),
            Register::Status => {
                return Err(EmulatorError::Bus {
                    address: u64::from(offset),
                    explanation: "I2C status is read-only".into(),
                });
            }
        }

        Ok(())
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::error::{HostError, HostResult};
use std::collections::BTreeMap;

use crate::machine::devices::{Device, DeviceActivity, InterruptLine};
use crate::machine::error::EmulatorError;

mod loopback_peripheral;
pub(crate) mod registers;
mod spi_peripheral;

pub(crate) use loopback_peripheral::LoopbackPeripheral;
use registers::{
    CHIP_SELECT_COUNT, CONTROL_INTERRUPT_ENABLE, DESELECTED, MODE_MASK,
    Register, STATUS_COMPLETE, STATUS_MISSING_PERIPHERAL,
};
pub(crate) use spi_peripheral::SpiPeripheral;

pub(crate) struct Spi {
    control: u32,
    host_error: Option<String>,
    mode: u8,
    peripherals: BTreeMap<u32, Box<dyn SpiPeripheral>>,
    response: u8,
    selected: u32,
    status: u32,
}

impl Spi {
    pub(crate) fn attach(
        &mut self,
        chip_select: u32,
        peripheral: Box<dyn SpiPeripheral>,
    ) -> Result<(), EmulatorError> {
        if chip_select >= CHIP_SELECT_COUNT
            || self.peripherals.contains_key(&chip_select)
        {
            return Err(EmulatorError::Configuration(
                "SPI chip select must be unique and less than 4".into(),
            ));
        }

        self.peripherals.insert(chip_select, peripheral);

        Ok(())
    }

    pub(crate) fn new() -> Self {
        Self {
            peripherals: BTreeMap::new(),
            selected: DESELECTED,
            mode: 0,
            control: 0,
            host_error: None,
            response: 0xff,
            status: 0,
        }
    }

    fn record_host_error<T>(
        &mut self,
        result: HostResult<T>,
    ) -> Result<T, EmulatorError> {
        result.map_err(|error| {
            let message = format!("SPI transport failed: {error}");
            self.host_error = Some(message.clone());
            EmulatorError::HostIo(HostError::other(message))
        })
    }

    pub(crate) fn replace_peripheral(
        &mut self,
        chip_select: u32,
        peripheral: Box<dyn SpiPeripheral>,
    ) -> Result<(), EmulatorError> {
        if chip_select >= CHIP_SELECT_COUNT || self.selected != DESELECTED {
            return Err(EmulatorError::Configuration(
                "SPI attachment requires select 0..3 and a deselected \
                 controller"
                    .into(),
            ));
        }
        self.peripherals.insert(chip_select, peripheral);
        Ok(())
    }
}

impl Default for Spi {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Spi {
    fn activity(&self) -> DeviceActivity {
        DeviceActivity {
            host_errors: true,
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
            Register::Data => u32::from(self.response),
            Register::Status => self.status,
            Register::ChipSelect => self.selected,
            Register::Mode => u32::from(self.mode),
            Register::Control => self.control,
            Register::Acknowledge => 0,
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
            Register::Data => {
                if let Some(peripheral) =
                    self.peripherals.get_mut(&self.selected)
                {
                    let response = peripheral.transfer(value as u8, self.mode);
                    self.response = self.record_host_error(response)?;
                    self.status = STATUS_COMPLETE;
                } else {
                    self.response = 0xff;
                    self.status = STATUS_COMPLETE | STATUS_MISSING_PERIPHERAL;
                }
            }
            Register::ChipSelect => {
                if value >= CHIP_SELECT_COUNT && value != DESELECTED {
                    return Err(EmulatorError::Bus {
                        address: u64::from(offset),
                        explanation: "invalid SPI chip select".into(),
                    });
                }

                if value != self.selected {
                    if let Some(peripheral) =
                        self.peripherals.get_mut(&self.selected)
                    {
                        let result = peripheral.select(false);
                        self.record_host_error(result)?;
                    }
                    if let Some(peripheral) = self.peripherals.get_mut(&value) {
                        let result = peripheral.select(true);
                        self.record_host_error(result)?;
                    }
                    self.selected = value;
                }
            }
            Register::Mode => self.mode = (value & MODE_MASK) as u8,
            Register::Control => {
                self.control = value & CONTROL_INTERRUPT_ENABLE
            }
            Register::Acknowledge => {
                if value & STATUS_COMPLETE != 0 {
                    self.status &= !STATUS_COMPLETE;
                }
            }
            Register::Status => {
                return Err(EmulatorError::Bus {
                    address: u64::from(offset),
                    explanation: "SPI register is read-only".into(),
                });
            }
        }

        Ok(())
    }
}

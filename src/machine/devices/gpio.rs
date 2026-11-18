// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::devices::{Device, DeviceActivity, InterruptLine};
use crate::machine::error::EmulatorError;
use crate::machine::error::HostError;

mod line_transport;
pub(crate) mod registers;

use registers::Register;

pub(crate) use line_transport::LineTransport;

#[derive(Default)]
pub(crate) struct Gpio {
    direction: u32,
    host: Option<Box<dyn LineTransport>>,
    host_error: Option<String>,
    inputs: u32,
    interrupt_mask: u32,
    output_latches: u32,
    pending: u32,
}

impl Gpio {
    fn check_host(&self) -> Result<(), EmulatorError> {
        if let Some(error) = &self.host_error {
            return Err(EmulatorError::HostIo(HostError::other(error.clone())));
        }

        Ok(())
    }

    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn outputs(&self) -> u32 {
        self.output_latches & self.direction
    }

    fn poll_host(&mut self) {
        if self.host_error.is_some() {
            return;
        }

        if let Some(host) = &mut self.host {
            match host.read() {
                Ok(values) => self.set_inputs(values),
                Err(error) => {
                    self.host_error =
                        Some(format!("GPIO input failed: {error}"))
                }
            }
        }
    }

    pub(crate) fn set_inputs(&mut self, values: u32) {
        let changed_to_high = !self.inputs & values & !self.direction;
        self.pending |= changed_to_high;
        self.inputs = values;
    }

    pub(crate) fn with_host(
        mut host: Box<dyn LineTransport>,
    ) -> Result<Self, EmulatorError> {
        host.write(0, 0)?;
        let inputs = host.read()?;

        Ok(Self {
            host: Some(host),
            inputs,
            ..Self::default()
        })
    }

    fn write_host(
        &mut self,
        direction: u32,
        outputs: u32,
    ) -> Result<(), EmulatorError> {
        if let Some(host) = &mut self.host
            && let Err(error) = host.write(direction, outputs)
        {
            self.host_error = Some(format!("GPIO output failed: {error}"));
            return self.check_host();
        }

        Ok(())
    }
}

impl Device for Gpio {
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
        (self.pending & self.interrupt_mask != 0)
            .then_some(InterruptLine::MachineExternal)
    }

    fn poll_host_inputs(&mut self) {
        self.poll_host();
    }

    fn read(
        &mut self,
        offset: u32,
        width: usize,
    ) -> Result<u32, EmulatorError> {
        let register = Register::access(offset, width)?;
        if matches!(register, Register::Pins | Register::Pending) {
            self.poll_host();
        }
        self.check_host()?;
        Ok(match register {
            Register::Pins => (self.inputs & !self.direction) | self.outputs(),
            Register::Output => self.output_latches,
            Register::Direction => self.direction,
            Register::InterruptMask => self.interrupt_mask,
            Register::Pending => self.pending,
        })
    }

    fn tick(&mut self, _instructions: u64) {
        self.poll_host();
    }

    fn write(
        &mut self,
        offset: u32,
        width: usize,
        value: u32,
    ) -> Result<(), EmulatorError> {
        let register = Register::access(offset, width)?;
        self.check_host()?;
        match register {
            Register::Output => {
                self.write_host(self.direction, value)?;
                self.output_latches = value;
            }
            Register::Direction => {
                self.write_host(value, self.output_latches)?;
                self.direction = value;
            }
            Register::InterruptMask => self.interrupt_mask = value,
            Register::Pending => self.pending &= !value,
            Register::Pins => {
                return Err(EmulatorError::Bus {
                    address: u64::from(offset),
                    explanation: "GPIO register is read-only".into(),
                });
            }
        }

        Ok(())
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::devices::{Device, DeviceActivity, InterruptLine};
use crate::machine::error::EmulatorError;

pub(crate) mod registers;

use registers::Register;
use registers::{NO_SOURCE, SOURCES};

#[derive(Default)]
pub(crate) struct InterruptController {
    enabled: u32,
    in_service: u32,
    latched: u32,
    levels: u32,
}

impl InterruptController {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn pending(&self) -> u32 {
        (self.levels | self.latched) & !self.in_service
    }

    pub(crate) fn set_levels(&mut self, levels: u32, edges: u32) {
        let levels = levels & !(1 << NO_SOURCE);
        self.latched |= levels & edges;
        self.levels = levels & !edges;
    }
}

impl Device for InterruptController {
    fn activity(&self) -> DeviceActivity {
        DeviceActivity {
            host_errors: false,
            interrupt_lines: &[InterruptLine::MachineExternal],
            ..DeviceActivity::default()
        }
    }

    fn interrupt(&self) -> Option<InterruptLine> {
        (self.pending() & self.enabled != 0)
            .then_some(InterruptLine::MachineExternal)
    }

    fn read(
        &mut self,
        offset: u32,
        width: usize,
    ) -> Result<u32, EmulatorError> {
        Ok(match Register::access(offset, width)? {
            Register::Pending => self.pending(),
            Register::Enabled => self.enabled,
            Register::ClaimComplete => {
                let pending = self.pending() & self.enabled;
                if pending == 0 {
                    NO_SOURCE
                } else {
                    let source = pending.trailing_zeros();
                    self.in_service |= 1 << source;
                    self.latched &= !(1 << source);
                    source
                }
            }
            Register::InService => self.in_service,
        })
    }

    fn write(
        &mut self,
        offset: u32,
        width: usize,
        value: u32,
    ) -> Result<(), EmulatorError> {
        match Register::access(offset, width)? {
            Register::Enabled => self.enabled = value & !(1 << NO_SOURCE),
            Register::ClaimComplete
                if (NO_SOURCE + 1..SOURCES).contains(&value)
                    && self.in_service & (1 << value) != 0 =>
            {
                self.in_service &= !(1 << value);
            }
            Register::Pending
            | Register::ClaimComplete
            | Register::InService => {
                return Err(EmulatorError::Bus {
                    address: u64::from(offset),
                    explanation: "read-only register or completion of an \
                                  unclaimed interrupt"
                        .into(),
                });
            }
        }

        Ok(())
    }
}

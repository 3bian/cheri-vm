// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::devices::{Device, DeviceActivity, InterruptLine};
use crate::machine::error::EmulatorError;

pub(crate) mod registers;

use registers::{
    CONTROL_ENABLE, CONTROL_INTERRUPT_ENABLE, PENDING_ACKNOWLEDGE, Register,
};

#[derive(Default)]
pub(crate) struct Timer32 {
    compare: u32,
    control: u32,
    count: u32,
    pending: bool,
}

impl Timer32 {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl Device for Timer32 {
    fn activity(&self) -> DeviceActivity {
        DeviceActivity {
            host_errors: false,
            interrupt_lines: &[InterruptLine::MachineTimer],
            ..DeviceActivity::default()
        }
    }

    fn interrupt(&self) -> Option<InterruptLine> {
        (self.pending && self.control & CONTROL_INTERRUPT_ENABLE != 0)
            .then_some(InterruptLine::MachineTimer)
    }

    fn read(
        &mut self,
        offset: u32,
        width: usize,
    ) -> Result<u32, EmulatorError> {
        Ok(match Register::access(offset, width)? {
            Register::Count => self.count,
            Register::Compare => self.compare,
            Register::Control => self.control,
            Register::Pending => u32::from(self.pending),
        })
    }

    fn tick(&mut self, ticks: u64) {
        if self.control & CONTROL_ENABLE == 0 {
            return;
        }

        let distance = self.compare.wrapping_sub(self.count);
        let distance = if distance == 0 {
            1_u64 << 32
        } else {
            u64::from(distance)
        };

        if ticks >= distance {
            self.pending = true;
        }

        self.count = self.count.wrapping_add(ticks as u32);
    }

    fn write(
        &mut self,
        offset: u32,
        width: usize,
        value: u32,
    ) -> Result<(), EmulatorError> {
        match Register::access(offset, width)? {
            Register::Count => self.count = value,
            Register::Compare => self.compare = value,
            Register::Control => {
                self.control =
                    value & (CONTROL_ENABLE | CONTROL_INTERRUPT_ENABLE)
            }
            Register::Pending => {
                if value & PENDING_ACKNOWLEDGE != 0 {
                    self.pending = false;
                }
            }
        }

        Ok(())
    }
}

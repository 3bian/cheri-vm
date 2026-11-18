// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::devices::{Device, DeviceActivity, InterruptLine};
use crate::machine::error::EmulatorError;

pub(crate) mod registers;

use registers::Register;
use registers::SOFTWARE_INTERRUPT_PENDING;

pub(crate) struct Timer64 {
    compare_value: u64,
    software_interrupt_pending: bool,
    time: u64,
    time_written: bool,
    timer_interrupt_pending: bool,
}

impl Timer64 {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn set_time(&mut self, time: u64) {
        self.time = time;
        self.timer_interrupt_pending = self.time >= self.compare_value;
    }

    pub(crate) fn take_written_time(&mut self) -> Option<u64> {
        std::mem::take(&mut self.time_written).then_some(self.time)
    }
}

impl Default for Timer64 {
    fn default() -> Self {
        Self {
            compare_value: 0,
            software_interrupt_pending: false,
            time: 0,
            time_written: false,
            timer_interrupt_pending: true,
        }
    }
}

impl Device for Timer64 {
    fn activity(&self) -> DeviceActivity {
        DeviceActivity {
            host_errors: false,
            interrupt_lines: &[
                InterruptLine::MachineSoftware,
                InterruptLine::MachineTimer,
            ],
            ..DeviceActivity::default()
        }
    }

    fn interrupt(&self) -> Option<InterruptLine> {
        if self.software_interrupt_pending {
            Some(InterruptLine::MachineSoftware)
        } else {
            self.timer_interrupt_pending
                .then_some(InterruptLine::MachineTimer)
        }
    }

    fn interrupt_pending(&self, line: InterruptLine) -> bool {
        match line {
            InterruptLine::MachineSoftware => self.software_interrupt_pending,
            InterruptLine::MachineTimer => self.timer_interrupt_pending,
            InterruptLine::MachineExternal
            | InterruptLine::SupervisorExternal => false,
        }
    }

    fn read(
        &mut self,
        offset: u32,
        width: usize,
    ) -> Result<u32, EmulatorError> {
        Ok(match Register::access(offset, width)? {
            Register::SoftwareInterrupt => {
                u32::from(self.software_interrupt_pending)
            }
            Register::CompareLow => self.compare_value as u32,
            Register::CompareHigh => (self.compare_value >> 32) as u32,
            Register::TimeLow => self.time as u32,
            Register::TimeHigh => (self.time >> 32) as u32,
        })
    }

    fn tick(&mut self, ticks: u64) {
        self.time = self.time.wrapping_add(ticks);
        self.timer_interrupt_pending = self.time >= self.compare_value;
    }

    fn write(
        &mut self,
        offset: u32,
        width: usize,
        value: u32,
    ) -> Result<(), EmulatorError> {
        match Register::access(offset, width)? {
            Register::SoftwareInterrupt => {
                self.software_interrupt_pending =
                    value & SOFTWARE_INTERRUPT_PENDING != 0;
            }
            Register::CompareLow => {
                self.compare_value = (self.compare_value
                    & 0xffff_ffff_0000_0000)
                    | u64::from(value)
            }
            Register::CompareHigh => {
                self.compare_value = (self.compare_value & 0xffff_ffff)
                    | (u64::from(value) << 32)
            }
            Register::TimeLow => {
                self.time =
                    (self.time & 0xffff_ffff_0000_0000) | u64::from(value);
                self.time_written = true;
            }
            Register::TimeHigh => {
                self.time =
                    (self.time & 0xffff_ffff) | (u64::from(value) << 32);
                self.time_written = true;
            }
        }

        self.timer_interrupt_pending = self.time >= self.compare_value;

        Ok(())
    }
}

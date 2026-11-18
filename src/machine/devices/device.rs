// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::device_activity::DeviceActivity;
use super::interrupt_line::InterruptLine;
use crate::machine::bus::MemoryPort;
use crate::machine::error::EmulatorError;
use std::any::Any;

pub(crate) trait Device: Any {
    fn activity(&self) -> DeviceActivity {
        DeviceActivity::default()
    }

    fn drain_output(&mut self, _output: &mut Vec<u8>) -> Option<&'static str> {
        None
    }

    fn drive_memory(&mut self, _ticks: u64, _memory: &mut MemoryPort<'_>) {}

    fn host_error(&self) -> Option<&str> {
        None
    }

    fn interrupt(&self) -> Option<InterruptLine> {
        None
    }

    fn interrupt_pending(&self, line: InterruptLine) -> bool {
        self.interrupt() == Some(line)
    }

    fn interrupt_taken(&mut self) {}

    fn poll_host_inputs(&mut self) {}

    fn read(&mut self, offset: u32, width: usize)
    -> Result<u32, EmulatorError>;

    fn tick(&mut self, _ticks: u64) {}

    fn write(
        &mut self,
        offset: u32,
        width: usize,
        value: u32,
    ) -> Result<(), EmulatorError>;
}

impl dyn Device {
    pub(crate) fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        (self as &mut dyn Any).downcast_mut::<T>()
    }

    pub(crate) fn downcast_ref<T: Any>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }

    pub(crate) fn interrupt_signature(&self) -> u8 {
        u8::from(self.interrupt_pending(InterruptLine::MachineExternal))
            | (u8::from(self.interrupt_pending(InterruptLine::MachineSoftware))
                << 1)
            | (u8::from(self.interrupt_pending(InterruptLine::MachineTimer))
                << 2)
            | (u8::from(
                self.interrupt_pending(InterruptLine::SupervisorExternal),
            ) << 3)
    }

    pub(crate) fn is<T: Any>(&self) -> bool {
        (self as &dyn Any).is::<T>()
    }
}

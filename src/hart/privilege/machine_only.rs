// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{Mode, PrivilegeModel, Reach, highest_interrupt};
use crate::hart::machine_registers::MachineRegisters;
use crate::isa::extension::ExtensionSet;
use crate::isa::xlen::Word;
use crate::machine::bus::Bus;
use crate::machine::error::{EmulatorError, ExceptionCause};

pub struct MachineOnly;

impl PrivilegeModel for MachineOnly {
    const SUPERVISOR: bool = false;

    fn enter_trap<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
        address: W,
        cause: u32,
        value: W,
        interrupt: bool,
    ) -> W {
        registers.enter_trap(address, cause, value, interrupt)
    }

    fn environment_call(&self) -> ExceptionCause {
        ExceptionCause::EnvironmentCall
    }

    fn fence<W: Word>(&mut self, _: &MachineRegisters<W>) -> bool {
        false
    }

    #[inline]
    fn interrupt<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
        lines: u32,
    ) -> Option<u32> {
        registers.interrupt_pending = lines;
        let enabled = lines & registers.interrupt_enable;
        if enabled == 0 || registers.machine_status.to_u64() & 8 == 0 {
            return None;
        }
        Some(highest_interrupt(enabled))
    }

    fn machine_return<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
    ) -> Option<W> {
        Some(registers.machine_return())
    }

    fn mode(&self) -> Mode {
        Mode::Machine
    }

    fn new<W: Word>(
        _: ExtensionSet,
        registers: &mut MachineRegisters<W>,
    ) -> Self {
        registers.machine_status = W::from_u64(3 << 11);
        Self
    }

    fn read_csr<W: Word>(
        &self,
        registers: &MachineRegisters<W>,
        address: u16,
    ) -> Option<W> {
        registers.read(address)
    }

    fn supervisor_return<W: Word>(
        &mut self,
        _: &mut MachineRegisters<W>,
    ) -> Option<W> {
        None
    }

    #[inline]
    fn translate<W: Word>(
        &mut self,
        _: &MachineRegisters<W>,
        _: &mut Bus,
        address: W,
        _: usize,
        _: Reach,
    ) -> Result<u64, ExceptionCause> {
        Ok(address.to_u64())
    }

    fn waits<W: Word>(&self, _: &MachineRegisters<W>) -> bool {
        true
    }

    fn write_csr<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
        address: u16,
        value: W,
    ) -> Result<(), EmulatorError> {
        registers.write(address, value)
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{Mode, Reach};
use crate::hart::machine_registers::MachineRegisters;
use crate::isa::extension::ExtensionSet;
use crate::isa::xlen::Word;
use crate::machine::bus::Bus;
use crate::machine::error::{EmulatorError, ExceptionCause};

pub trait PrivilegeModel: Sized + 'static {
    const SUPERVISOR: bool;

    fn capability_load<W: Word>(
        &self,
        _registers: &MachineRegisters<W>,
        tagged: bool,
    ) -> Result<bool, ExceptionCause> {
        Ok(tagged)
    }

    fn capability_store<W: Word>(
        &self,
        _registers: &MachineRegisters<W>,
    ) -> Result<(), ExceptionCause> {
        Ok(())
    }

    fn enter_trap<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
        address: W,
        cause: u32,
        value: W,
        interrupt: bool,
    ) -> W;

    fn environment_call(&self) -> ExceptionCause;

    fn fence<W: Word>(&mut self, registers: &MachineRegisters<W>) -> bool;

    fn inspect_address<W: Word>(
        &self,
        _registers: &MachineRegisters<W>,
        _bus: &Bus,
        address: u64,
    ) -> Option<u64> {
        Some(address)
    }

    fn inspect_csr<W: Word>(
        &self,
        registers: &MachineRegisters<W>,
        address: u16,
    ) -> Option<W> {
        self.read_csr(registers, address)
    }

    fn interrupt<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
        lines: u32,
    ) -> Option<u32>;

    fn machine_return<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
    ) -> Option<W>;

    fn mode(&self) -> Mode;

    fn new<W: Word>(
        extensions: ExtensionSet,
        registers: &mut MachineRegisters<W>,
    ) -> Self;

    fn read_csr<W: Word>(
        &self,
        registers: &MachineRegisters<W>,
        address: u16,
    ) -> Option<W>;

    fn supervisor_return<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
    ) -> Option<W>;

    fn translate<W: Word>(
        &mut self,
        registers: &MachineRegisters<W>,
        bus: &mut Bus,
        address: W,
        width: usize,
        reach: Reach,
    ) -> Result<u64, ExceptionCause>;

    fn waits<W: Word>(&self, registers: &MachineRegisters<W>) -> bool;

    fn write_csr<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
        address: u16,
        value: W,
    ) -> Result<(), EmulatorError>;
}

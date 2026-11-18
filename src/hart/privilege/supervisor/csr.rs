// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{
    EXECUTABLE_READABLE, GENERATIONS_ENABLE, MACHINE_ENABLE, MACHINE_PREVIOUS,
    MACHINE_PREVIOUS_ENABLE, MODIFY_PRIVILEGE, Mode, SUPERVISOR_ENABLE,
    SUPERVISOR_GENERATION, SUPERVISOR_PREVIOUS, SUPERVISOR_PREVIOUS_ENABLE,
    SUPERVISOR_USER_MEMORY, Supervisor, TIMEOUT_WAIT, TRAP_SUPERVISOR_RETURN,
    TRAP_VIRTUAL_MEMORY, USER_GENERATION,
};
use crate::hart::machine_registers::{
    FLOAT_STATE, MachineRegisters, with_summary,
};
use crate::isa::xlen::Word;
use crate::machine::error::EmulatorError;

pub(super) const CAPABILITY_EXCEPTIONS: u64 = 0x17 << 32;
pub(super) const DELEGABLE_EXCEPTIONS: u64 = 0xb3ff;
pub(super) const GENERATION_EXCEPTION: u64 = 1 << 35;
const INTERRUPTS: u32 = 0xaaa;

const MACHINE_STATUS: u64 = SUPERVISOR_STATUS
    | MACHINE_ENABLE
    | MACHINE_PREVIOUS_ENABLE
    | MACHINE_PREVIOUS
    | MODIFY_PRIVILEGE
    | TRAP_VIRTUAL_MEMORY
    | TIMEOUT_WAIT
    | TRAP_SUPERVISOR_RETURN;

const SUPERVISOR_INTERRUPTS: u32 = 0x222;

const SUPERVISOR_STATUS: u64 = SUPERVISOR_ENABLE
    | SUPERVISOR_PREVIOUS_ENABLE
    | SUPERVISOR_PREVIOUS
    | SUPERVISOR_USER_MEMORY
    | EXECUTABLE_READABLE;

const WIDTHS: u64 = (2 << 32) | (2 << 34);

impl Supervisor {
    fn counter_hidden(&self, address: u16) -> bool {
        let bit = 1 << (address & 31);
        (self.mode != Mode::Machine && self.machine_counter_enable & bit == 0)
            || (self.mode == Mode::User
                && self.registers.counter_enable & bit == 0)
    }

    fn generation_bits(&self) -> u64 {
        if self.generations {
            GENERATIONS_ENABLE | SUPERVISOR_GENERATION | USER_GENERATION
        } else {
            0
        }
    }

    pub(super) fn inspect<W: Word>(
        &self,
        registers: &MachineRegisters<W>,
        address: u16,
    ) -> Option<W> {
        let status = registers.machine_status.to_u64();
        let delegated = self.delegated_interrupts;
        let value = match address {
            0x100 => {
                let seen =
                    SUPERVISOR_STATUS | FLOAT_STATE | self.generation_bits();
                with_summary(status & seen, 64) | (2 << 32)
            }
            0x104 => u64::from(registers.interrupt_enable & delegated),
            0x105 => self.registers.trap_vector,
            0x106 => u64::from(self.registers.counter_enable),
            0x140 => self.registers.scratch,
            0x141 => self.exception_address(registers),
            0x142 => self.registers.cause,
            0x143 => self.registers.trap_value,
            0x144 => u64::from(registers.interrupt_pending & delegated),
            0x180 => self.translation.root(),
            0x300 => with_summary(status, 64) | WIDTHS,
            0x302 => self.delegated_exceptions,
            0x303 => u64::from(delegated),
            0x306 => u64::from(self.machine_counter_enable),
            0x3a0 => self.pmp.configuration(),
            0x3b0..=0x3bf => self.pmp.address(usize::from(address - 0x3b0)),
            0xc01 => return None,
            0x10a
            | 0x30a
            | 0x3a2
            | 0x323..=0x33f
            | 0x7a0..=0x7a3
            | 0xb03..=0xb1f
            | 0xc03..=0xc1f => 0,
            _ => return registers.read(address),
        };
        Some(W::from_u64(value))
    }

    pub(super) fn read<W: Word>(
        &self,
        registers: &MachineRegisters<W>,
        address: u16,
    ) -> Option<W> {
        if (address >> 8) & 3 > self.mode as u16
            || (address == 0x180 && self.virtual_memory_trapped(registers))
            || (matches!(address, 0xc00 | 0xc02..=0xc1f)
                && self.counter_hidden(address))
        {
            return None;
        }
        self.inspect(registers, address)
    }

    fn set_software_pending<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
        writable: u32,
        value: u32,
    ) {
        self.software_pending =
            (self.software_pending & !writable) | (value & writable);
        registers.interrupt_pending = (registers.interrupt_pending
            & !SUPERVISOR_INTERRUPTS)
            | self.software_pending;
    }

    fn virtual_memory_trapped<W: Word>(
        &self,
        registers: &MachineRegisters<W>,
    ) -> bool {
        self.mode == Mode::Supervisor
            && registers.machine_status.to_u64() & TRAP_VIRTUAL_MEMORY != 0
    }

    pub(super) fn write<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
        address: u16,
        value: u64,
    ) -> Result<(), EmulatorError> {
        if address >> 10 == 3 || (address >> 8) & 3 > self.mode as u16 {
            return Err(refused(address));
        }
        let status = registers.machine_status.to_u64();
        let delegated = self.delegated_interrupts;
        let low = value as u32;
        match address {
            0x100 => {
                let kept = SUPERVISOR_STATUS
                    | registers.float_state()
                    | self.generation_bits();
                registers.machine_status =
                    W::from_u64((status & !kept) | (value & kept));
            }
            0x104 => {
                registers.interrupt_enable = (registers.interrupt_enable
                    & !delegated)
                    | (low & delegated);
            }
            0x105 => self.registers.trap_vector = vector(value),
            0x106 => self.registers.counter_enable = low,
            0x140 => self.registers.scratch = value,
            0x141 => self.registers.exception_program_counter = value & !1,
            0x142 => self.registers.cause = value,
            0x143 => self.registers.trap_value = value,
            0x144 => self.set_software_pending(registers, delegated & 2, low),
            0x180 if self.virtual_memory_trapped(registers) => {
                return Err(refused(address));
            }
            0x180 => self.translation.set_root(value),
            0x300 => {
                let kept = MACHINE_STATUS
                    | registers.float_state()
                    | self.generation_bits();
                let mut next = (status & !kept) | (value & kept);
                if next & MACHINE_PREVIOUS == 2 << 11 {
                    next = (next & !MACHINE_PREVIOUS)
                        | (status & MACHINE_PREVIOUS);
                }
                registers.machine_status = W::from_u64(next);
            }
            0x302 => {
                self.delegated_exceptions = value & self.delegable_exceptions
            }
            0x303 => self.delegated_interrupts = low & SUPERVISOR_INTERRUPTS,
            0x304 => registers.interrupt_enable = low & INTERRUPTS,
            0x306 => self.machine_counter_enable = low,
            0x344 => {
                self.set_software_pending(registers, SUPERVISOR_INTERRUPTS, low)
            }
            0x3a0 => self.pmp.set_configuration(value),
            0x3b0..=0x3bf => {
                self.pmp.set_address(usize::from(address - 0x3b0), value)
            }
            0x10a
            | 0x30a
            | 0x3a2
            | 0x323..=0x33f
            | 0x7a0..=0x7a3
            | 0xb03..=0xb1f => {}
            _ => return registers.write(address, W::from_u64(value)),
        }
        Ok(())
    }
}

fn refused(address: u16) -> EmulatorError {
    EmulatorError::Refused(format!(
        "CSR 0x{address:03x} is not writable or implemented"
    ))
}

fn vector(value: u64) -> u64 {
    (value & !3) | u64::from(value & 3 == 1)
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod supervisor_registers;

pub(crate) use supervisor_registers::SupervisorRegisters;

mod csr;

use csr::{CAPABILITY_EXCEPTIONS, DELEGABLE_EXCEPTIONS, GENERATION_EXCEPTION};

use super::pmp::Pmp;

use super::{Mode, PrivilegeModel, Reach, highest_interrupt};

use crate::hart::machine_registers::MachineRegisters;

use crate::hart::translation::{
    CAPABILITY_ENTRY, CAPABILITY_FIELD, Translation,
};

use crate::isa::extension::ExtensionSet;

use crate::isa::xlen::Word;

use crate::machine::bus::Bus;

use crate::machine::error::{EmulatorError, ExceptionCause};

pub(crate) const EXECUTABLE_READABLE: u64 = 1 << 19;
pub(crate) const GENERATIONS_ENABLE: u64 = 1 << 59;
pub(crate) const MACHINE_ENABLE: u64 = 1 << 3;
pub(crate) const MACHINE_PREVIOUS: u64 = 3 << 11;
pub(crate) const MACHINE_PREVIOUS_ENABLE: u64 = 1 << 7;
pub(crate) const MODIFY_PRIVILEGE: u64 = 1 << 17;
pub(crate) const SUPERVISOR_ENABLE: u64 = 1 << 1;
pub(crate) const SUPERVISOR_GENERATION: u64 = 1 << 60;
pub(crate) const SUPERVISOR_PREVIOUS: u64 = 1 << 8;
pub(crate) const SUPERVISOR_PREVIOUS_ENABLE: u64 = 1 << 5;
pub(crate) const SUPERVISOR_USER_MEMORY: u64 = 1 << 18;
pub(crate) const TIMEOUT_WAIT: u64 = 1 << 21;
pub(crate) const TRAP_SUPERVISOR_RETURN: u64 = 1 << 22;
pub(crate) const TRAP_VIRTUAL_MEMORY: u64 = 1 << 20;
pub(crate) const USER_GENERATION: u64 = 1 << 61;

pub struct Supervisor {
    delegable_exceptions: u64,
    delegated_exceptions: u64,
    delegated_interrupts: u32,
    generations: bool,
    machine_counter_enable: u32,
    mode: Mode,
    pmp: Pmp,
    registers: SupervisorRegisters,
    software_pending: u32,
    translation: Translation,
}

impl Supervisor {
    fn exception_address<W: Word>(
        &self,
        registers: &MachineRegisters<W>,
    ) -> u64 {
        self.registers.exception_program_counter
            & !(u64::from(registers.instruction_alignment.max(2)) - 1)
    }

    fn tracks_generations(&self, status: u64) -> bool {
        self.generations && status & GENERATIONS_ENABLE != 0
    }
}

impl PrivilegeModel for Supervisor {
    const SUPERVISOR: bool = true;

    fn capability_load<W: Word>(
        &self,
        registers: &MachineRegisters<W>,
        tagged: bool,
    ) -> Result<bool, ExceptionCause> {
        let Some((field, user)) = self.translation.last else {
            return Ok(tagged);
        };
        let status = registers.machine_status.to_u64();
        if !self.tracks_generations(status) {
            return Ok(tagged && field & 8 != 0);
        }
        let generation = field & 2 != 0;
        if field & 1 == 0 {
            return Ok(tagged && generation);
        }
        let current = if user {
            USER_GENERATION
        } else {
            SUPERVISOR_GENERATION
        };
        if tagged && generation != (status & current != 0) {
            return Err(ExceptionCause::CheriLoadCapabilityFault);
        }
        Ok(tagged)
    }

    fn capability_store<W: Word>(
        &self,
        registers: &MachineRegisters<W>,
    ) -> Result<(), ExceptionCause> {
        let needed = match self.translation.last {
            None => return Ok(()),
            Some(_)
                if self
                    .tracks_generations(registers.machine_status.to_u64()) =>
            {
                0xc
            }
            Some(_) => 8,
        };
        if self
            .translation
            .last
            .is_some_and(|(field, _)| field & needed == needed)
        {
            Ok(())
        } else {
            Err(ExceptionCause::CheriStorePageFault)
        }
    }

    fn enter_trap<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
        address: W,
        cause: u32,
        value: W,
        interrupt: bool,
    ) -> W {
        let delegations = if interrupt {
            u64::from(self.delegated_interrupts)
        } else {
            self.delegated_exceptions
        };
        let tagged = u64::from(cause) | u64::from(interrupt) << 63;
        let status = registers.machine_status.to_u64();
        let target =
            if self.mode != Mode::Machine && delegations >> cause & 1 == 1 {
                self.registers.cause = tagged;
                self.registers.exception_program_counter = address.to_u64();
                self.registers.trap_value = value.to_u64();
                registers.machine_status = W::from_u64(
                    (status
                        & !(SUPERVISOR_ENABLE
                            | SUPERVISOR_PREVIOUS_ENABLE
                            | SUPERVISOR_PREVIOUS))
                        | (status & SUPERVISOR_ENABLE) << 4
                        | if self.mode == Mode::Supervisor {
                            SUPERVISOR_PREVIOUS
                        } else {
                            0
                        },
                );
                self.mode = Mode::Supervisor;
                self.registers.trap_vector
            } else {
                registers.exception_program_counter = address;
                registers.trap_cause = W::from_u64(tagged);
                registers.trap_value = value;
                registers.machine_status = W::from_u64(
                    (status
                        & !(MACHINE_ENABLE
                            | MACHINE_PREVIOUS_ENABLE
                            | MACHINE_PREVIOUS))
                        | (status & MACHINE_ENABLE) << 4
                        | (self.mode as u64) << 11,
                );
                self.mode = Mode::Machine;
                registers.trap_vector.to_u64()
            };
        W::from_u64(vector(target, cause, interrupt))
    }

    fn environment_call(&self) -> ExceptionCause {
        match self.mode {
            Mode::User => ExceptionCause::EnvironmentCallFromUser,
            Mode::Supervisor => ExceptionCause::EnvironmentCallFromSupervisor,
            Mode::Machine => ExceptionCause::EnvironmentCall,
        }
    }

    fn fence<W: Word>(&mut self, registers: &MachineRegisters<W>) -> bool {
        let trapped =
            registers.machine_status.to_u64() & TRAP_VIRTUAL_MEMORY != 0;
        if self.mode == Mode::User || (self.mode == Mode::Supervisor && trapped)
        {
            return false;
        }
        self.translation.flush();
        true
    }

    fn inspect_address<W: Word>(
        &self,
        _: &MachineRegisters<W>,
        bus: &Bus,
        address: u64,
    ) -> Option<u64> {
        if self.mode == Mode::Machine || !self.translation.enabled() {
            Some(address)
        } else {
            self.translation.inspect(bus, address)
        }
    }

    fn inspect_csr<W: Word>(
        &self,
        registers: &MachineRegisters<W>,
        address: u16,
    ) -> Option<W> {
        self.inspect(registers, address)
    }

    #[inline]
    fn interrupt<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
        lines: u32,
    ) -> Option<u32> {
        let pending = self.software_pending | lines;
        registers.interrupt_pending = pending;
        let enabled = pending & registers.interrupt_enable;
        if enabled == 0 {
            return None;
        }
        let status = registers.machine_status.to_u64();
        let machine = enabled & !self.delegated_interrupts;
        if machine != 0
            && (self.mode != Mode::Machine || status & MACHINE_ENABLE != 0)
        {
            return Some(highest_interrupt(machine));
        }
        let supervisor = enabled & self.delegated_interrupts;
        let open = match self.mode {
            Mode::User => true,
            Mode::Supervisor => status & SUPERVISOR_ENABLE != 0,
            Mode::Machine => false,
        };
        (supervisor != 0 && open).then(|| highest_interrupt(supervisor))
    }

    fn machine_return<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
    ) -> Option<W> {
        if self.mode != Mode::Machine {
            return None;
        }
        let status = registers.machine_status.to_u64();
        let previous = previous_mode(status);
        let mut next = (status & !(MACHINE_ENABLE | MACHINE_PREVIOUS))
            | (status & MACHINE_PREVIOUS_ENABLE) >> 4
            | MACHINE_PREVIOUS_ENABLE;
        if previous != Mode::Machine {
            next &= !MODIFY_PRIVILEGE;
        }
        registers.machine_status = W::from_u64(next);
        self.mode = previous;
        Some(registers.exception_address())
    }

    fn mode(&self) -> Mode {
        self.mode
    }

    fn new<W: Word>(
        extensions: ExtensionSet,
        registers: &mut MachineRegisters<W>,
    ) -> Self {
        registers.machine_status = W::default();
        let capabilities = extensions.contains(ExtensionSet::Y);
        let generations = extensions.contains(ExtensionSet::SVYRG);
        let mut delegable_exceptions = DELEGABLE_EXCEPTIONS;
        if capabilities {
            delegable_exceptions |= CAPABILITY_EXCEPTIONS;
        }
        if generations {
            delegable_exceptions |= GENERATION_EXCEPTION;
        }
        Self {
            delegable_exceptions,
            delegated_exceptions: 0,
            delegated_interrupts: 0,
            generations,
            machine_counter_enable: 0,
            mode: Mode::Machine,
            pmp: Pmp::default(),
            registers: SupervisorRegisters::default(),
            software_pending: 0,
            translation: Translation::with_capability_field(
                match (capabilities, generations) {
                    (_, true) => CAPABILITY_FIELD,
                    (true, false) => CAPABILITY_ENTRY,
                    (false, false) => 0,
                },
            ),
        }
    }

    fn read_csr<W: Word>(
        &self,
        registers: &MachineRegisters<W>,
        address: u16,
    ) -> Option<W> {
        self.read(registers, address)
    }

    fn supervisor_return<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
    ) -> Option<W> {
        let status = registers.machine_status.to_u64();
        let trapped = status & TRAP_SUPERVISOR_RETURN != 0;
        if self.mode == Mode::User || (self.mode == Mode::Supervisor && trapped)
        {
            return None;
        }
        self.mode = if status & SUPERVISOR_PREVIOUS != 0 {
            Mode::Supervisor
        } else {
            Mode::User
        };
        registers.machine_status = W::from_u64(
            (status
                & !(SUPERVISOR_ENABLE
                    | SUPERVISOR_PREVIOUS
                    | MODIFY_PRIVILEGE))
                | (status & SUPERVISOR_PREVIOUS_ENABLE) >> 4
                | SUPERVISOR_PREVIOUS_ENABLE,
        );
        Some(W::from_u64(self.exception_address(registers)))
    }

    #[inline]
    fn translate<W: Word>(
        &mut self,
        registers: &MachineRegisters<W>,
        bus: &mut Bus,
        address: W,
        width: usize,
        reach: Reach,
    ) -> Result<u64, ExceptionCause> {
        let status = registers.machine_status.to_u64();
        let mode = if reach != Reach::Fetch && status & MODIFY_PRIVILEGE != 0 {
            previous_mode(status)
        } else {
            self.mode
        };
        let physical = if mode == Mode::Machine || !self.translation.enabled() {
            self.translation.last = None;
            address.to_u64()
        } else {
            self.translation.resolve(
                bus,
                &self.pmp,
                address.to_u64(),
                reach,
                mode,
                status,
            )?
        };
        if self.pmp.allows(physical, width, reach, mode) {
            Ok(physical)
        } else {
            Err(reach.access_fault())
        }
    }

    fn waits<W: Word>(&self, registers: &MachineRegisters<W>) -> bool {
        match self.mode {
            Mode::User => false,
            Mode::Supervisor => {
                registers.machine_status.to_u64() & TIMEOUT_WAIT == 0
            }
            Mode::Machine => true,
        }
    }

    fn write_csr<W: Word>(
        &mut self,
        registers: &mut MachineRegisters<W>,
        address: u16,
        value: W,
    ) -> Result<(), EmulatorError> {
        self.write(registers, address, value.to_u64())
    }
}

fn previous_mode(status: u64) -> Mode {
    match (status & MACHINE_PREVIOUS) >> 11 {
        0 => Mode::User,
        1 => Mode::Supervisor,
        _ => Mode::Machine,
    }
}

fn vector(base: u64, cause: u32, interrupt: bool) -> u64 {
    if interrupt && base & 3 == 1 {
        (base & !3).wrapping_add(4 * u64::from(cause))
    } else {
        base & !3
    }
}

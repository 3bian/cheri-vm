// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod metadata;
mod shadow;

pub use metadata::Metadata;
pub use shadow::Shadow;

mod control;

mod execute;

mod memory;

use super::Model;

use crate::hart::Hart;

use crate::hart::plain::{Plain, Processor};

use crate::hart::privilege::PrivilegeModel;

use crate::isa::extension::ExtensionSet;

use crate::isa::instruction::Instruction;

use crate::isa::pointer::rvy::{Capability128, Window, permissions};

use crate::isa::pointer::{Access, PointerModel, Rvy};

use crate::isa::xlen::Rv64;

use crate::machine::error::{EmulatorError, ExceptionCause};

impl<P: PrivilegeModel> Processor<Rv64, P, Rvy> {
    pub fn capability(&self, index: u8) -> Capability128 {
        if index == 0 {
            Capability128::default()
        } else {
            self.shadow.registers[usize::from(index)]
                .at(self.read_register_unchecked(index))
        }
    }

    pub fn program_counter_capability(&self) -> Capability128 {
        self.shadow.program_counter.at(self.program_counter)
    }

    pub fn set_capability(&mut self, index: u8, capability: Capability128) {
        self.write_register_unchecked(index, capability.address);
        if index != 0 {
            self.shadow.registers[usize::from(index)] =
                Metadata::of(capability);
        }
    }
}

impl Model<Rv64> for Rvy {
    const CAPABILITIES: bool = true;
    type Pointer = Self;
    type Shadow = Shadow;

    #[inline]
    fn authority(shadow: &Shadow, _: u64) -> Window {
        shadow.window
    }

    fn call(
        shadow: &mut Shadow,
        destination: u8,
        base: Option<(u8, u64, i32)>,
        next: u64,
        target: u64,
    ) {
        let current = shadow.program_counter;
        let entered = match base {
            Some((0, _, _)) => Capability128::integer(target),
            Some((register, address, offset)) => {
                let source =
                    shadow.registers[usize::from(register)].at(address);
                if source.sealed() && offset == 0 && address & 1 == 0 {
                    source.with_seal(false).with_address(target)
                } else {
                    source.with_address(target)
                }
            }
            None if shadow.window.covers(target) => {
                if destination != 0 {
                    let link = current.at(next).with_seal(true);
                    shadow.registers[usize::from(destination)] =
                        Metadata::of(link);
                }
                return;
            }
            None => current.at(next).with_address(target),
        };
        if destination != 0 {
            let link = current.at(next).with_seal(true);
            shadow.registers[usize::from(destination)] = Metadata::of(link);
        }
        shadow.enter(entered);
    }

    fn check(
        shadow: &Shadow,
        base: u8,
        target: u64,
        width: usize,
        access: Access,
    ) -> Result<(), ExceptionCause> {
        let authority = if base == 0 {
            Capability128::default()
        } else {
            shadow.registers[usize::from(base)].at(target)
        };
        Rvy::check_access(authority, target, width, access).map_err(|_| {
            if access.writes() {
                ExceptionCause::CheriStoreAccessFault
            } else {
                ExceptionCause::CheriLoadAccessFault
            }
        })
    }

    #[inline]
    fn clear(shadow: &mut Shadow, register: u8) {
        shadow.registers[usize::from(register)] = Metadata::default();
    }

    fn control<P: PrivilegeModel>(
        hart: &mut Hart<Plain<Rv64, P, Self>>,
        instruction: Instruction,
    ) -> Option<Result<(), EmulatorError>> {
        hart.capability_control(instruction)
    }

    fn execute<P: PrivilegeModel>(
        hart: &mut Hart<Plain<Rv64, P, Self>>,
        instruction: Instruction,
        following: u64,
    ) -> Result<u64, EmulatorError> {
        match instruction {
            Instruction::Rvy { .. } => hart.capability(instruction),
            _ => hart.capability_memory(instruction),
        }?;
        Ok(following)
    }

    #[inline]
    fn jump(shadow: &mut Shadow, from: u64, target: u64) {
        if !shadow.window.covers(target) {
            shadow.enter(shadow.program_counter.at(from).with_address(target));
        }
    }

    fn permits_system(shadow: &Shadow) -> bool {
        shadow.program_counter.at(0).permissions() & permissions::SYSTEM != 0
    }

    fn returned(shadow: &mut Shadow, supervisor: bool, target: u64) {
        let saved = shadow.exception_program_counters[usize::from(supervisor)]
            .at(target);
        shadow.enter(saved.with_seal(false));
    }

    fn shadow(extensions: ExtensionSet) -> Shadow {
        let levels = extensions.contains(ExtensionSet::ZYLG);
        let root = Capability128::infinite(levels);
        Shadow {
            exception_program_counters: [Metadata::of(root); 2],
            levels,
            program_counter: Metadata::of(root),
            registers: [Metadata::default(); 32],
            scratches: [Metadata::default(); 2],
            thread_identifiers: [Capability128::default(); 3],
            trap_vectors: [Metadata::of(root); 2],
            window: Window::of(root),
        }
    }

    fn trap(shadow: &mut Shadow, supervisor: bool, vector: u64, handler: u64) {
        let mode = usize::from(supervisor);
        shadow.exception_program_counters[mode] = shadow.program_counter;
        let entered =
            shadow.trap_vectors[mode].at(vector).with_address(handler);
        shadow.enter(entered);
    }

    fn upper(shadow: &mut Shadow, destination: u8, from: u64, value: u64) {
        if destination != 0 {
            let derived = shadow.program_counter.at(from).with_address(value);
            shadow.registers[usize::from(destination)] = Metadata::of(derived);
        }
    }
}

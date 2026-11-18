// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{Metadata, Shadow};
use crate::hart::Hart;
use crate::hart::plain::Plain;
use crate::hart::plain::model::Model;
use crate::hart::privilege::{Mode, PrivilegeModel};
use crate::isa::extension::rvy::csr;
use crate::isa::instruction::{CsrOperation, Instruction};
use crate::isa::pointer::Rvy;
use crate::isa::pointer::rvy::Capability128;
use crate::isa::xlen::Rv64;
use crate::machine::error::EmulatorError;

impl<P: PrivilegeModel> Hart<Plain<Rv64, P, Rvy>> {
    pub(super) fn capability_control(
        &mut self,
        instruction: Instruction,
    ) -> Option<Result<(), EmulatorError>> {
        let system = Rvy::permits_system(&self.processor.shadow);
        match instruction {
            Instruction::MachineReturn | Instruction::SupervisorReturn
                if !system =>
            {
                Some(Err(refused("a trap return")))
            }
            Instruction::Csr {
                operation,
                destination,
                source,
                address,
                immediate,
            } => {
                let listed = csr::CAPABILITY_REGISTERS.contains(&address);
                let thread = listed && csr::is_thread_identifier(address);
                let guarded = if thread {
                    operation.writes(source)
                } else {
                    address >> 8 & 3 != 0
                };
                if guarded && !system {
                    return Some(Err(refused("a system register")));
                }
                if !listed {
                    return None;
                }
                let written = operation.writes(source).then(|| {
                    let whole =
                        operation == CsrOperation::ReadWrite && !immediate;
                    let given = if immediate {
                        Capability128::integer(source.into())
                    } else {
                        self.processor.capability(source)
                    };
                    (whole, given)
                });
                let previous = if thread {
                    Ok(self.thread_identifier(address, operation, written))
                } else {
                    self.widened(address, operation, written)
                };
                match previous {
                    Ok(Some(previous)) => {
                        self.processor.set_capability(destination, previous);
                        Some(Ok(()))
                    }
                    Ok(None) => None,
                    Err(error) => Some(Err(error)),
                }
            }
            _ => None,
        }
    }

    fn thread_identifier(
        &mut self,
        address: u16,
        operation: CsrOperation,
        written: Written,
    ) -> Option<Capability128> {
        let level = |mode| match mode {
            Mode::User => 0,
            Mode::Supervisor => 1,
            Mode::Machine => 2,
        };
        let (index, needed) = match address {
            csr::USER_THREAD_IDENTIFIER => (0, 0),
            csr::SUPERVISOR_THREAD_IDENTIFIER if P::SUPERVISOR => (1, 1),
            csr::MACHINE_THREAD_IDENTIFIER => (2, 2),
            _ => return None,
        };
        if level(self.processor.privilege.mode()) < needed {
            return None;
        }
        let slot = &mut self.processor.shadow.thread_identifiers[index];
        let previous = *slot;
        if let Some((whole, given)) = written {
            *slot = derived(previous, operation, whole, given);
        }
        Some(previous)
    }

    fn widened(
        &mut self,
        address: u16,
        operation: CsrOperation,
        written: Written,
    ) -> Result<Option<Capability128>, EmulatorError> {
        let processor = &mut self.processor;
        let Some(held) = processor
            .privilege
            .read_csr::<u64>(&processor.machine_registers, address)
        else {
            return Ok(None);
        };
        let vector = address & 0xff == 5;
        let previous = slot(&mut processor.shadow, address).at(held);
        if let Some((whole, given)) = written {
            let next = derived(previous, operation, whole, given);
            processor.privilege.write_csr(
                &mut processor.machine_registers,
                address,
                next.address,
            )?;
            let stored = processor
                .privilege
                .read_csr::<u64>(&processor.machine_registers, address);
            *slot(&mut processor.shadow, address) = Metadata {
                bits: next.metadata,
                tag: next.tag
                    && stored == Some(next.address)
                    && !(vector && next.sealed()),
            };
        }
        Ok(Some(previous))
    }
}

type Written = Option<(bool, Capability128)>;

fn derived(
    previous: Capability128,
    operation: CsrOperation,
    whole: bool,
    given: Capability128,
) -> Capability128 {
    if whole {
        Capability128 {
            tag: given.tag && given.is_sound(),
            ..given
        }
    } else {
        previous.with_address(operation.apply(previous.address, given.address))
    }
}

fn refused(what: &str) -> EmulatorError {
    EmulatorError::Refused(format!("{what} needs the system permission"))
}

fn slot(shadow: &mut Shadow, address: u16) -> &mut Metadata {
    let supervisor = usize::from(address >> 8 & 3 == 1);
    match address & 0xff {
        5 => &mut shadow.trap_vectors[supervisor],
        0x41 => &mut shadow.exception_program_counters[supervisor],
        _ => &mut shadow.scratches[supervisor],
    }
}

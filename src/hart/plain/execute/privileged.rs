// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::Hart;
use crate::hart::plain::Plain;
use crate::hart::plain::model::Model;
use crate::hart::privilege::PrivilegeModel;
use crate::isa::instruction::Instruction;
use crate::isa::xlen::{Word, Xlen};
use crate::machine::error::{EmulatorError, ExceptionCause};

impl<X: Xlen, P: PrivilegeModel, M: Model<X>> Hart<Plain<X, P, M>> {
    pub(super) fn privileged(
        &mut self,
        instruction: Instruction,
        address: X::Word,
        following: X::Word,
    ) -> Result<X::Word, EmulatorError> {
        use Instruction::*;
        if M::CAPABILITIES
            && let Some(result) = M::control(self, instruction)
        {
            return result.map(|()| following);
        }
        let processor = &mut self.processor;
        let registers = &mut processor.machine_registers;
        match instruction {
            WaitForInterrupt => {
                if !processor.privilege.waits(registers) {
                    return Err(refused("wfi"));
                }
                self.timing.wait_for_interrupt(
                    &mut self.bus,
                    registers,
                    &mut self.host_clock,
                );
                Ok(following)
            }
            EnvironmentCall => Err(EmulatorError::Trap {
                cause: processor.privilege.environment_call(),
                address: address.to_u64(),
                value: 0,
            }),
            Breakpoint => Err(EmulatorError::Trap {
                cause: ExceptionCause::Breakpoint,
                address: address.to_u64(),
                value: 0,
            }),
            MachineReturn | SupervisorReturn => {
                let supervisor = instruction == SupervisorReturn;
                let target = if supervisor {
                    processor.privilege.supervisor_return(registers)
                } else {
                    processor.privilege.machine_return(registers)
                }
                .ok_or_else(|| {
                    refused(if supervisor { "sret" } else { "mret" })
                })?;
                M::returned(&mut processor.shadow, supervisor, target);
                Ok(target)
            }
            SupervisorFence => {
                if !processor.privilege.fence(registers) {
                    return Err(refused("sfence.vma"));
                }
                Ok(following)
            }
            Csr {
                operation,
                destination,
                source,
                address: csr_address,
                immediate,
            } => {
                let floating = (1..=3).contains(&csr_address);
                let previous: X::Word = if floating {
                    processor
                        .float_csr(csr_address)
                        .map(|value| X::Word::from_u64(u64::from(value)))
                } else {
                    processor.privilege.read_csr(registers, csr_address)
                }
                .ok_or_else(|| {
                    EmulatorError::Refused(format!(
                        "CSR 0x{csr_address:03x} is unsupported"
                    ))
                })?;
                let source_value = if immediate {
                    X::Word::from_u64(u64::from(source))
                } else {
                    processor.read_register_unchecked(source)
                };
                if operation.writes(source) {
                    let new_value = operation.apply(previous, source_value);
                    if floating {
                        processor.set_float_csr(
                            csr_address,
                            new_value.to_u64() as u32,
                        );
                    } else {
                        processor.privilege.write_csr(
                            &mut processor.machine_registers,
                            csr_address,
                            new_value,
                        )?;
                    }
                }
                processor.write_register_unchecked(destination, previous);
                Ok(following)
            }
            _ => Err(EmulatorError::Execution(
                "not a privileged instruction".to_owned(),
            )),
        }
    }
}

fn refused(what: &str) -> EmulatorError {
    EmulatorError::Refused(format!("{what} is not allowed in this mode"))
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::cheriot_rv32e::{CheriotRv32e, Processor};
use crate::hart::{Fault, Hart};
use crate::isa::instruction::Instruction;

impl Hart<CheriotRv32e> {
    pub(super) fn privileged(
        &mut self,
        instruction: Instruction,
        next: u32,
    ) -> Result<u32, Fault> {
        let mut following = next;
        match instruction {
            Instruction::Breakpoint => {
                return Err(Fault {
                    cause: 3,
                    value: self.processor.program_counter,
                });
            }
            Instruction::EnvironmentCall => {
                return Err(Fault {
                    cause: 11,
                    value: 0,
                });
            }
            Instruction::MachineReturn => {
                self.require_system_registers(Processor::PCC)?;
                if self.processor.machine_registers.machine_status & (3 << 11)
                    != 3 << 11
                {
                    return Err(Fault::illegal());
                }
                self.processor.machine_registers.machine_return();
                self.processor.pcc = self.processor.machine_exception;
                following = self.processor.machine_exception.address();
            }
            Instruction::WaitForInterrupt => {
                self.timing.wait_for_interrupt(
                    &mut self.bus,
                    &mut self.processor.machine_registers,
                    &mut self.host_clock,
                );
            }
            Instruction::Csr {
                operation,
                destination,
                source,
                address,
                immediate,
            } => {
                if destination >= 16 {
                    return Err(Fault::illegal());
                }
                let value = if immediate {
                    u32::from(source)
                } else {
                    self.processor.read_register(source)?.address()
                };
                let writing = operation.writes(source);
                Self::check_csr_access(address, writing)?;
                let counter_read = !writing
                    && matches!(address, 0xb00 | 0xb02 | 0xb80 | 0xb82);
                if !counter_read {
                    self.require_system_registers(Processor::PCC)?;
                }
                let previous = match address {
                    0x7a0 => !self.processor.trigger_select,
                    0xbc0 => 3,
                    0xbc1 => self.processor.stack_high_water,
                    0xbc2 => self.processor.stack_base,
                    0x305 => self.processor.machine_trap.address(),
                    0x341 => self.processor.machine_exception.address(),
                    _ => self
                        .processor
                        .machine_registers
                        .read(address)
                        .ok_or_else(Fault::illegal)?,
                };
                if writing {
                    let written = operation.apply(previous, value);
                    match address {
                        0x304 => {
                            self.processor.machine_registers.interrupt_enable =
                                written & 0xaaa
                        }
                        0x344 => {
                            self.processor
                                .machine_registers
                                .interrupt_pending = (self
                                .processor
                                .machine_registers
                                .interrupt_pending
                                & !0x222)
                                | (written & 0x222);
                        }
                        0x300 => {
                            let mask = 0x7f_ffff
                                & !(0x44 | (3 << 15) | (1 << 17) | 0x11);
                            let mut status = written & mask;
                            if status & (3 << 13) == 3 << 13
                                || status & (3 << 9) == 3 << 9
                            {
                                status |= 1 << 31;
                            }
                            self.processor.machine_registers.machine_status =
                                status;
                        }
                        0xbc0 => (),
                        0xbc1 => {
                            self.processor.stack_high_water = written & !15
                        }
                        0xbc2 => self.processor.stack_base = written & !15,
                        0x7a0 => self.processor.trigger_select = written,
                        0x305 | 0x341 => return Err(Fault::illegal()),
                        _ => self
                            .processor
                            .machine_registers
                            .write(address, written)
                            .map_err(|_| Fault::illegal())?,
                    }
                }
                self.processor
                    .write_integer_register(destination, previous)?;
            }
            _ => return Err(Fault::illegal()),
        }
        Ok(following)
    }
}

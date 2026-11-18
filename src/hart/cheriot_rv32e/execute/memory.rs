// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::cheriot_rv32e::CheriotRv32e;
#[cfg(feature = "verification")]
use crate::hart::cheriot_rv32e::MemoryAccess;
use crate::hart::{Fault, Hart};
use crate::isa::instruction::Instruction;
use crate::isa::pointer::Access;

impl Hart<CheriotRv32e> {
    pub(super) fn memory(
        &mut self,
        instruction: Instruction,
        next: u32,
    ) -> Result<u32, Fault> {
        let following = next;
        match instruction {
            Instruction::Load {
                kind,
                destination,
                base,
                offset,
            } => {
                if destination >= 16 {
                    return Err(Fault::illegal());
                }
                let address = self
                    .processor
                    .read_register(base)?
                    .address()
                    .wrapping_add(offset as u32);
                let width = kind.width();
                self.check_memory_access(base, address, width, Access::Load)?;
                let bits =
                    self.bus.read(address.into(), width).map_err(|_| {
                        Fault {
                            cause: 5,
                            value: address,
                        }
                    })?;
                #[cfg(feature = "verification")]
                self.trace.push(MemoryAccess::Read {
                    address,
                    width,
                    value: u64::from(bits),
                    tag: None,
                });
                let value = kind.extend(u64::from(bits));
                self.processor.write_integer_register(destination, value)?;
            }
            Instruction::Store {
                kind,
                source,
                base,
                offset,
            } => {
                let value = self.processor.read_register(source)?.address();
                let address = self
                    .processor
                    .read_register(base)?
                    .address()
                    .wrapping_add(offset as u32);
                let width = kind.width();
                self.check_memory_access(base, address, width, Access::Store)?;
                self.bus.write(address.into(), width, value).map_err(|_| {
                    Fault {
                        cause: 7,
                        value: address,
                    }
                })?;
                self.record_store(address, width);
                #[cfg(feature = "verification")]
                self.trace.push(MemoryAccess::Write {
                    address,
                    width,
                    value: u64::from(value) & ((1_u64 << (width * 8)) - 1),
                    tag: None,
                });
            }
            Instruction::Fence
            | Instruction::FenceInstruction
            | Instruction::Hint => (),
            _ => return Err(Fault::illegal()),
        }
        Ok(following)
    }

    pub(super) fn record_store(&mut self, address: u32, width: usize) {
        self.last_store = Some((address.into(), width));
        if address >= self.processor.stack_base
            && address < self.processor.stack_high_water
        {
            self.processor.stack_high_water = address & !15;
        }
    }
}

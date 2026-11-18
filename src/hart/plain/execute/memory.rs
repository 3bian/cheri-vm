// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::Hart;
use crate::hart::plain::Plain;
use crate::hart::plain::model::Model;
use crate::hart::privilege::{PrivilegeModel, Reach};
use crate::isa::instruction::Instruction;
use crate::isa::pointer::{Access, Integer, PointerModel};
use crate::isa::xlen::{Word, Xlen};
use crate::machine::error::{EmulatorError, ExceptionCause};

impl<X: Xlen, P: PrivilegeModel, M: Model<X>> Hart<Plain<X, P, M>> {
    pub(in crate::hart::plain) fn capability_load(
        &mut self,
        address: X::Word,
        reach: Reach,
        tagged: bool,
    ) -> Result<bool, EmulatorError> {
        self.reach(address, 16, reach)?;
        let processor = &self.processor;
        processor
            .privilege
            .capability_load(&processor.machine_registers, tagged)
            .map_err(|cause| EmulatorError::Trap {
                cause,
                address: processor.program_counter.to_u64(),
                value: address.to_u64(),
            })
    }

    pub(in crate::hart::plain) fn capability_store(
        &mut self,
        address: X::Word,
    ) -> Result<(), EmulatorError> {
        self.reach(address, 16, Reach::Store)?;
        let processor = &self.processor;
        processor
            .privilege
            .capability_store(&processor.machine_registers)
            .map_err(|cause| EmulatorError::Trap {
                cause,
                address: processor.program_counter.to_u64(),
                value: address.to_u64(),
            })
    }

    pub(super) fn check_alignment(
        &self,
        address: X::Word,
        base: u8,
        target: X::Word,
        width: usize,
        access: Access,
    ) -> Result<(), EmulatorError> {
        M::check(&self.processor.shadow, base, target, width, access).map_err(
            |cause| EmulatorError::Trap {
                cause,
                address: address.to_u64(),
                value: target.to_u64(),
            },
        )?;
        Integer::<X::Word>::check_access(target, target, width, access).map_err(
            |_| EmulatorError::Trap {
                cause: if access.writes() {
                    ExceptionCause::StoreAddressMisaligned
                } else {
                    ExceptionCause::LoadAddressMisaligned
                },
                address: address.to_u64(),
                value: target.to_u64(),
            },
        )
    }

    pub(in crate::hart::plain) fn load(
        &mut self,
        address: X::Word,
        width: usize,
        reach: Reach,
    ) -> Result<u64, EmulatorError> {
        let physical = self.reach(address, width, reach)?;
        let read = if width < 8 {
            self.bus.read(physical, width).map(u64::from)
        } else {
            self.bus.read(physical, 4).and_then(|low| {
                let high = self.bus.read(physical.wrapping_add(4), 4)?;
                Ok(u64::from(low) | u64::from(high) << 32)
            })
        };
        read.map_err(|error| at(error, address))
    }

    pub(super) fn memory(
        &mut self,
        instruction: Instruction,
        address: X::Word,
        following: X::Word,
    ) -> Result<X::Word, EmulatorError> {
        use Instruction::*;
        let read = |number| self.processor.read_register_unchecked(number);
        match instruction {
            Load {
                kind,
                destination,
                base,
                offset,
            } => {
                let load_address = read(base).add_signed(offset);
                let width = kind.width();
                self.check_alignment(
                    address,
                    base,
                    load_address,
                    width,
                    Access::Load,
                )?;
                let value = self.load(load_address, width, Reach::Load)?;
                let result = kind.extend(value);
                self.processor.write_register_unchecked(destination, result);
                Ok(following)
            }
            Store {
                kind,
                source,
                base,
                offset,
            } => {
                let store_address = read(base).add_signed(offset);
                let width = kind.width();
                self.check_alignment(
                    address,
                    base,
                    store_address,
                    width,
                    Access::Store,
                )?;
                let value = read(source);
                self.store(store_address, width, value.to_u64())?;
                Ok(following)
            }
            Fence | FenceInstruction | Hint => Ok(following),
            _ => Err(EmulatorError::Execution(
                "not a memory instruction".to_owned(),
            )),
        }
    }

    pub(in crate::hart::plain) fn memory_tag(
        &mut self,
        address: X::Word,
    ) -> Result<bool, EmulatorError> {
        let physical = self.reach(address, 1, Reach::Load)?;
        Ok(self.bus.tag_at(physical))
    }

    #[inline]
    fn reach(
        &mut self,
        address: X::Word,
        width: usize,
        reach: Reach,
    ) -> Result<u64, EmulatorError> {
        self.processor
            .privilege
            .translate(
                &self.processor.machine_registers,
                &mut self.bus,
                address,
                width,
                reach,
            )
            .map_err(|cause| EmulatorError::Trap {
                cause,
                address: 0,
                value: address.to_u64(),
            })
    }

    pub(in crate::hart::plain) fn set_memory_tag(
        &mut self,
        address: X::Word,
    ) -> Result<(), EmulatorError> {
        let physical = self.reach(address, 1, Reach::Store)?;
        self.bus.set_tag(physical);
        Ok(())
    }

    pub(in crate::hart::plain) fn store(
        &mut self,
        address: X::Word,
        width: usize,
        value: u64,
    ) -> Result<(), EmulatorError> {
        let physical = self.reach(address, width, Reach::Store)?;
        let written = if width < 8 {
            self.bus.write(physical, width, value as u32)
        } else {
            self.bus.write(physical, 4, value as u32).and_then(|()| {
                self.bus.write(
                    physical.wrapping_add(4),
                    4,
                    (value >> 32) as u32,
                )
            })
        };
        written.map_err(|error| at(error, address))?;
        self.last_store = Some((physical, width));
        Ok(())
    }
}

fn at<W: Word>(error: EmulatorError, address: W) -> EmulatorError {
    match error {
        EmulatorError::Bus { explanation, .. } => EmulatorError::Bus {
            address: address.to_u64(),
            explanation,
        },
        other => other,
    }
}

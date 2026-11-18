// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::Hart;
use crate::hart::plain::Plain;
use crate::hart::plain::model::Model;
use crate::hart::privilege::{PrivilegeModel, Reach};
use crate::isa::instruction::{AtomicOperation, AtomicWidth, Instruction};
use crate::isa::pointer::Access;
use crate::isa::xlen::{Word, Xlen};
use crate::machine::error::EmulatorError;

impl<X: Xlen, P: PrivilegeModel, M: Model<X>> Hart<Plain<X, P, M>> {
    pub(super) fn atomic(
        &mut self,
        instruction: Instruction,
        address: X::Word,
        following: X::Word,
    ) -> Result<X::Word, EmulatorError> {
        let Instruction::Atomic {
            operation,
            width,
            destination,
            address: base,
            source,
        } = instruction
        else {
            return Err(EmulatorError::Execution(
                "not an atomic instruction".to_owned(),
            ));
        };
        let target = self.processor.read_register_unchecked(base);
        let source = self.processor.read_register_unchecked(source).to_u64();
        let bytes = width.bytes();
        let extend = |value: u64| match width {
            AtomicWidth::Capability | AtomicWidth::Doubleword => {
                X::Word::from_u64(value)
            }
            AtomicWidth::Word => X::Word::from_i32(value as i32),
        };
        let result = match operation {
            AtomicOperation::LoadReserved => {
                self.check_alignment(
                    address,
                    base,
                    target,
                    bytes,
                    Access::Load,
                )?;
                let value = self.load(target, bytes, Reach::Load)?;
                self.processor.reservation = Some(target.to_u64());
                extend(value)
            }
            AtomicOperation::StoreConditional => {
                self.check_alignment(
                    address,
                    base,
                    target,
                    bytes,
                    Access::Store,
                )?;
                let held =
                    self.processor.reservation.take() == Some(target.to_u64());
                if held {
                    self.store(target, bytes, source)?;
                }
                X::Word::from_u64(u64::from(!held))
            }
            operation => {
                self.check_alignment(
                    address,
                    base,
                    target,
                    bytes,
                    Access::Load,
                )?;
                self.check_alignment(
                    address,
                    base,
                    target,
                    bytes,
                    Access::Store,
                )?;
                let value = self.load(target, bytes, Reach::Store)?;
                let combined = match width {
                    AtomicWidth::Capability | AtomicWidth::Doubleword => {
                        combine(operation, value, source)
                    }
                    AtomicWidth::Word => u64::from(combine(
                        operation,
                        value as u32,
                        source as u32,
                    )),
                };
                self.store(target, bytes, combined)?;
                extend(value)
            }
        };
        self.processor.write_register_unchecked(destination, result);
        Ok(following)
    }
}

fn combine<W: Word>(operation: AtomicOperation, held: W, source: W) -> W {
    use AtomicOperation::*;
    match operation {
        Add => held.add(source),
        And => held & source,
        Maximum if held.to_i64() < source.to_i64() => source,
        MaximumUnsigned if held < source => source,
        Minimum if held.to_i64() > source.to_i64() => source,
        MinimumUnsigned if held > source => source,
        Or => held | source,
        Swap => source,
        Xor => (held | source) & !(held & source),
        LoadReserved | Maximum | MaximumUnsigned | Minimum
        | MinimumUnsigned | StoreConditional => held,
    }
}

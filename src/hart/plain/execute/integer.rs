// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::Hart;
use crate::hart::plain::Plain;
use crate::hart::plain::model::Model;
use crate::hart::privilege::PrivilegeModel;
use crate::isa::arithmetic::{
    perform_aes_byte_operation, perform_immediate_operation,
    perform_register_operation,
};
use crate::isa::instruction::Instruction;
use crate::isa::xlen::{Word, Xlen};
use crate::machine::error::{EmulatorError, ExceptionCause};

impl<X: Xlen, P: PrivilegeModel, M: Model<X>> Hart<Plain<X, P, M>> {
    pub(super) fn checked_target(
        &self,
        address: X::Word,
    ) -> Result<X::Word, EmulatorError> {
        let alignment =
            u64::from(self.processor.extensions.instruction_alignment());
        if address.to_u64() & (alignment - 1) != 0 {
            return Err(EmulatorError::Trap {
                cause: ExceptionCause::InstructionAddressMisaligned,
                address: self.processor.program_counter.to_u64(),
                value: address.to_u64(),
            });
        }

        Ok(address)
    }

    pub(super) fn integer(
        &mut self,
        instruction: Instruction,
        address: X::Word,
        following: X::Word,
    ) -> Result<X::Word, EmulatorError> {
        use Instruction::*;
        let read = |number| self.processor.read_register_unchecked(number);
        let low = |number| {
            self.processor.read_register_unchecked(number).to_u64() as u32
        };
        match instruction {
            Register {
                operation,
                destination,
                first,
                second,
            } => {
                let first_value = read(first);
                let second_value = read(second);
                let result = perform_register_operation(
                    operation,
                    first_value,
                    second_value,
                );
                self.processor.write_register_unchecked(destination, result);
                Ok(following)
            }
            RegisterWord {
                operation,
                destination,
                first,
                second,
            } => {
                let result = perform_register_operation(
                    operation,
                    low(first),
                    low(second),
                );
                self.processor.write_register_unchecked(
                    destination,
                    X::Word::from_i32(result as i32),
                );
                Ok(following)
            }
            Immediate {
                operation,
                destination,
                source,
                immediate,
            } => {
                let first_value = read(source);
                let result = perform_immediate_operation(
                    operation,
                    first_value,
                    immediate,
                );
                self.processor.write_register_unchecked(destination, result);
                Ok(following)
            }
            ImmediateWord {
                operation,
                destination,
                source,
                immediate,
            } => {
                let result = perform_immediate_operation(
                    operation,
                    low(source),
                    immediate,
                );
                self.processor.write_register_unchecked(
                    destination,
                    X::Word::from_i32(result as i32),
                );
                Ok(following)
            }
            AesByte {
                operation,
                destination,
                first,
                second,
                select,
            } => {
                let result = perform_aes_byte_operation(
                    operation,
                    low(first),
                    low(second),
                    select,
                );
                self.processor.write_register_unchecked(
                    destination,
                    X::Word::from_i32(result as i32),
                );
                Ok(following)
            }
            Branch {
                condition,
                first,
                second,
                offset,
            } => {
                let first_value = read(first);
                let second_value = read(second);
                let taken = condition.test(first_value, second_value);
                if taken {
                    let target =
                        self.checked_target(address.add_signed(offset))?;
                    M::jump(&mut self.processor.shadow, address, target);
                    Ok(target)
                } else {
                    Ok(following)
                }
            }
            JumpAndLink {
                destination,
                offset,
            } => {
                let target = address.add_signed(offset);
                self.link(destination, None, following, target)
            }
            JumpAndLinkRegister {
                destination,
                base,
                offset,
            } => {
                let held = read(base);
                let target =
                    X::Word::from_u64(held.add_signed(offset).to_u64() & !1);
                let through = Some((base, held, offset));
                self.link(destination, through, following, target)
            }
            LoadUpperImmediate { destination, value } => {
                self.processor.write_register_unchecked(
                    destination,
                    X::Word::from_i32(value as i32),
                );
                Ok(following)
            }
            AddUpperImmediateToProgramCounter { destination, value } => {
                let derived = address.add_signed(value as i32);
                self.processor.write_address(destination, derived);
                M::upper(
                    &mut self.processor.shadow,
                    destination,
                    address,
                    derived,
                );
                Ok(following)
            }
            _ => Err(EmulatorError::Execution(
                "not an integer instruction".to_owned(),
            )),
        }
    }

    #[inline(always)]
    fn link(
        &mut self,
        destination: u8,
        through: Option<(u8, X::Word, i32)>,
        following: X::Word,
        target: X::Word,
    ) -> Result<X::Word, EmulatorError> {
        let target = self.checked_target(target)?;
        self.processor.write_address(destination, following);
        M::call(
            &mut self.processor.shadow,
            destination,
            through,
            following,
            target,
        );
        Ok(target)
    }
}

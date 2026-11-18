// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::cheriot_rv32e::CheriotRv32e;
use crate::hart::{Fault, Hart};
use crate::isa::arithmetic::{
    perform_immediate_operation, perform_register_operation,
};
use crate::isa::instruction::Instruction;

impl Hart<CheriotRv32e> {
    pub(super) fn integer(
        &mut self,
        instruction: Instruction,
        next: u32,
    ) -> Result<u32, Fault> {
        let mut following = next;
        match instruction {
            Instruction::Register {
                operation,
                destination,
                first,
                second,
            } => {
                let value = perform_register_operation(
                    operation,
                    self.processor.read_register(first)?.address(),
                    self.processor.read_register(second)?.address(),
                );
                self.processor.write_integer_register(destination, value)?;
            }
            Instruction::Immediate {
                operation,
                destination,
                source,
                immediate,
            } => {
                let value = perform_immediate_operation(
                    operation,
                    self.processor.read_register(source)?.address(),
                    immediate,
                );
                self.processor.write_integer_register(destination, value)?;
            }
            Instruction::LoadUpperImmediate { destination, value } => {
                self.processor.write_integer_register(destination, value)?
            }
            Instruction::Branch {
                condition,
                first,
                second,
                offset,
            } => {
                let first = self.processor.read_register(first)?.address();
                let second = self.processor.read_register(second)?.address();
                let taken = condition.test(first, second);
                if taken {
                    following = self
                        .processor
                        .program_counter
                        .wrapping_add(offset as u32);
                }
            }
            _ => return Err(Fault::illegal()),
        }
        Ok(following)
    }
}

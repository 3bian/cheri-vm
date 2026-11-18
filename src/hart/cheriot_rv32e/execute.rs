// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::CheriotRv32e;
use crate::hart::{Fault, Hart};
use crate::isa::decode::DecodedInstruction;
use crate::isa::extension::cheriot::CapabilityInstruction;
use crate::isa::instruction::Instruction;

mod capability;
mod checks;
mod integer;
mod memory;
mod privileged;

impl Hart<CheriotRv32e> {
    pub(super) fn execute(
        &mut self,
        decoded: DecodedInstruction,
    ) -> Result<(), Fault> {
        let next = self
            .processor
            .program_counter
            .wrapping_add(decoded.length());

        match decoded.instruction {
            Instruction::Capability {
                operation,
                registers,
                second,
                immediate,
            } => self.capability(
                CapabilityInstruction {
                    destination: registers.destination(),
                    first: registers.first(),
                    immediate,
                    operation,
                    second,
                },
                next,
            ),
            instruction => self.execute_integer(instruction, next),
        }
    }

    fn execute_integer(
        &mut self,
        instruction: Instruction,
        next: u32,
    ) -> Result<(), Fault> {
        self.processor.program_counter = match instruction {
            instruction @ (Instruction::Register { .. }
            | Instruction::Immediate { .. }
            | Instruction::LoadUpperImmediate { .. }
            | Instruction::AddUpperImmediateToProgramCounter {
                ..
            }
            | Instruction::JumpAndLink { .. }
            | Instruction::JumpAndLinkRegister { .. }
            | Instruction::Branch { .. }) => self.integer(instruction, next),
            instruction @ (Instruction::Load { .. }
            | Instruction::Store { .. }
            | Instruction::Fence
            | Instruction::FenceInstruction
            | Instruction::Hint) => self.memory(instruction, next),
            instruction => self.privileged(instruction, next),
        }?;
        Ok(())
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Plain;
use crate::hart::plain::model::Model;
use crate::hart::privilege::PrivilegeModel;
use crate::hart::{Architecture, Exception, Fault, Hart};
use crate::isa::decode::DecodedInstruction;
use crate::isa::instruction::{
    AtomicOperation, AtomicWidth, Instruction, LoadKind, StoreKind,
};
use crate::isa::xlen::{Word, Xlen};
use crate::machine::error::{EmulatorError, ExceptionCause};

mod atomic;
mod float;
mod integer;
mod memory;
mod privileged;

impl<X: Xlen, P: PrivilegeModel, M: Model<X>> Hart<Plain<X, P, M>> {
    fn dispatch(
        &mut self,
        instruction: Instruction,
        length: u32,
    ) -> Result<X::Word, EmulatorError> {
        use Instruction::*;
        let address = self.processor.program_counter;
        let following = address.add_signed(length as i32);

        match instruction {
            instruction @ (Rvy { .. }
            | Load {
                kind: LoadKind::Capability,
                ..
            }
            | Store {
                kind: StoreKind::Capability,
                ..
            }
            | Atomic {
                width: AtomicWidth::Capability,
                ..
            }) => M::execute(self, instruction, following),
            instruction @ (Register { .. }
            | RegisterWord { .. }
            | Immediate { .. }
            | ImmediateWord { .. }
            | AesByte { .. }
            | Branch { .. }
            | JumpAndLink { .. }
            | JumpAndLinkRegister { .. }
            | LoadUpperImmediate { .. }
            | AddUpperImmediateToProgramCounter { .. }) => {
                self.integer(instruction, address, following)
            }
            instruction @ (Load { .. }
            | Store { .. }
            | Fence
            | FenceInstruction
            | Hint) => self.memory(instruction, address, following),
            instruction @ Atomic { .. } => {
                self.atomic(instruction, address, following)
            }
            instruction @ (Float { .. }
            | FloatLoad { .. }
            | FloatStore { .. }) => self.float(instruction, address, following),
            instruction => self.privileged(instruction, address, following),
        }
    }

    pub(super) fn execute(
        &mut self,
        decoded: DecodedInstruction,
    ) -> Result<(), Exception<X::Word>> {
        let illegal = Plain::<X, P, M>::illegal_instruction(decoded.encoded);
        if self.processor.extensions.register_count() < 32
            && !self.valid_registers(decoded.instruction)
        {
            return Err(illegal.into());
        }
        let fault = match self.dispatch(decoded.instruction, decoded.length()) {
            Ok(next) => {
                self.processor.program_counter = next;
                return Ok(());
            }
            Err(EmulatorError::Trap { cause, value, .. }) => Fault {
                cause: cause.code(),
                value: X::Word::from_u64(value),
            },
            Err(EmulatorError::Bus { address, .. }) => Fault {
                cause: match decoded.instruction {
                    Instruction::Load { .. }
                    | Instruction::FloatLoad { .. }
                    | Instruction::Atomic {
                        operation: AtomicOperation::LoadReserved,
                        ..
                    } => ExceptionCause::LoadAccessFault,
                    Instruction::Store { .. }
                    | Instruction::FloatStore { .. }
                    | Instruction::Atomic { .. } => {
                        ExceptionCause::StoreAccessFault
                    }
                    _ => ExceptionCause::InstructionAccessFault,
                }
                .code(),
                value: X::Word::from_u64(address),
            },
            Err(EmulatorError::Refused(_)) => illegal,
            Err(error) => return Err(Exception::Fatal(error)),
        };
        Err(fault.into())
    }

    fn valid_registers(&self, instruction: Instruction) -> bool {
        let valid = |number| self.processor.validate_register(number);
        match instruction {
            Instruction::Register {
                destination,
                first,
                second,
                ..
            }
            | Instruction::RegisterWord {
                destination,
                first,
                second,
                ..
            }
            | Instruction::AesByte {
                destination,
                first,
                second,
                ..
            }
            | Instruction::Atomic {
                destination,
                address: first,
                source: second,
                ..
            } => valid(destination) && valid(first) && valid(second),
            Instruction::Immediate {
                destination,
                source,
                ..
            }
            | Instruction::ImmediateWord {
                destination,
                source,
                ..
            } => valid(destination) && valid(source),
            Instruction::Load {
                destination, base, ..
            }
            | Instruction::JumpAndLinkRegister {
                destination, base, ..
            } => valid(destination) && valid(base),
            Instruction::Store { source, base, .. } => {
                valid(source) && valid(base)
            }
            Instruction::Branch { first, second, .. } => {
                valid(first) && valid(second)
            }
            Instruction::JumpAndLink { destination, .. }
            | Instruction::LoadUpperImmediate { destination, .. }
            | Instruction::AddUpperImmediateToProgramCounter {
                destination,
                ..
            } => valid(destination),
            Instruction::Csr {
                destination,
                source,
                immediate,
                ..
            } => valid(destination) && (immediate || valid(source)),
            _ => true,
        }
    }
}

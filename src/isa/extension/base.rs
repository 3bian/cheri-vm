// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::decode::sign_extend;
use crate::isa::instruction::{
    BranchCondition, ImmediateOperation, Instruction, LoadKind,
    RegisterOperation, StoreKind,
};

#[allow(clippy::too_many_lines)]
pub fn standard(word: u32, xlen: u32) -> Option<Instruction> {
    let wide = xlen == 64;
    let opcode = word & 0x7f;
    let destination = ((word >> 7) & 31) as u8;
    let function_code = (word >> 12) & 7;
    let first = ((word >> 15) & 31) as u8;
    let second = ((word >> 20) & 31) as u8;
    let extended_function_code = (word >> 25) & 0x7f;
    let signed_immediate = sign_extend(word >> 20, 12);
    let instruction = match opcode {
        0x37 => Instruction::LoadUpperImmediate {
            destination,
            value: word & 0xfffff000,
        },
        0x17 => Instruction::AddUpperImmediateToProgramCounter {
            destination,
            value: word & 0xfffff000,
        },
        0x6f => {
            let immediate = ((word >> 31) << 20)
                | (((word >> 12) & 0xff) << 12)
                | (((word >> 20) & 1) << 11)
                | (((word >> 21) & 0x3ff) << 1);
            Instruction::JumpAndLink {
                destination,
                offset: sign_extend(immediate, 21),
            }
        }
        0x67 if function_code == 0 => Instruction::JumpAndLinkRegister {
            destination,
            base: first,
            offset: signed_immediate,
        },
        0x63 => {
            let condition = match function_code {
                0 => BranchCondition::Equal,
                1 => BranchCondition::NotEqual,
                4 => BranchCondition::LessThan,
                5 => BranchCondition::GreaterOrEqual,
                6 => BranchCondition::LessThanUnsigned,
                7 => BranchCondition::GreaterOrEqualUnsigned,
                _ => return None,
            };
            let immediate = ((word >> 31) << 12)
                | (((word >> 7) & 1) << 11)
                | (((word >> 25) & 0x3f) << 5)
                | (((word >> 8) & 15) << 1);
            Instruction::Branch {
                condition,
                first,
                second,
                offset: sign_extend(immediate, 13),
            }
        }
        0x03 => {
            let kind = match function_code {
                0 => LoadKind::SignedByte,
                1 => LoadKind::SignedHalfword,
                2 => LoadKind::Word,
                3 if wide => LoadKind::Doubleword,
                4 => LoadKind::UnsignedByte,
                5 => LoadKind::UnsignedHalfword,
                6 if wide => LoadKind::UnsignedWord,
                _ => return None,
            };
            Instruction::Load {
                kind,
                destination,
                base: first,
                offset: signed_immediate,
            }
        }
        0x23 => {
            let kind = match function_code {
                0 => StoreKind::Byte,
                1 => StoreKind::Halfword,
                2 => StoreKind::Word,
                3 if wide => StoreKind::Doubleword,
                _ => return None,
            };
            let immediate = ((word >> 25) << 5) | ((word >> 7) & 31);
            Instruction::Store {
                kind,
                source: second,
                base: first,
                offset: sign_extend(immediate, 12),
            }
        }
        0x13 => {
            let shift_code = if wide {
                (word >> 26) << 1
            } else {
                extended_function_code
            };
            let operation = match function_code {
                0 => ImmediateOperation::Add,
                2 => ImmediateOperation::SetLessThan,
                3 => ImmediateOperation::SetLessThanUnsigned,
                4 => ImmediateOperation::Xor,
                6 => ImmediateOperation::Or,
                7 => ImmediateOperation::And,
                1 if shift_code == 0 => ImmediateOperation::ShiftLeft,
                5 if shift_code == 0 => ImmediateOperation::ShiftRightLogical,
                5 if shift_code == 0x20 => {
                    ImmediateOperation::ShiftRightArithmetic
                }
                _ => return None,
            };
            Instruction::Immediate {
                operation,
                destination,
                source: first,
                immediate: if matches!(function_code, 1 | 5) {
                    ((word >> 20) & (xlen - 1)) as i32
                } else {
                    signed_immediate
                },
            }
        }
        0x1b if wide => {
            let operation = match (function_code, extended_function_code) {
                (0, _) => ImmediateOperation::Add,
                (1, 0) => ImmediateOperation::ShiftLeft,
                (5, 0) => ImmediateOperation::ShiftRightLogical,
                (5, 0x20) => ImmediateOperation::ShiftRightArithmetic,
                _ => return None,
            };
            Instruction::ImmediateWord {
                operation,
                destination,
                source: first,
                immediate: if function_code == 0 {
                    signed_immediate
                } else {
                    i32::from(second)
                },
            }
        }
        0x3b if wide => {
            let operation = match (extended_function_code, function_code) {
                (0, 0) => RegisterOperation::Add,
                (0x20, 0) => RegisterOperation::Sub,
                (0, 1) => RegisterOperation::ShiftLeft,
                (0, 5) => RegisterOperation::ShiftRightLogical,
                (0x20, 5) => RegisterOperation::ShiftRightArithmetic,
                _ => return None,
            };
            Instruction::RegisterWord {
                operation,
                destination,
                first,
                second,
            }
        }
        0x33 => {
            use RegisterOperation::*;
            let operation = match (extended_function_code, function_code) {
                (0, 0) => Add,
                (0x20, 0) => Sub,
                (0, 1) => ShiftLeft,
                (0, 2) => SetLessThan,
                (0, 3) => SetLessThanUnsigned,
                (0, 4) => Xor,
                (0, 5) => ShiftRightLogical,
                (0x20, 5) => ShiftRightArithmetic,
                (0, 6) => Or,
                (0, 7) => And,
                _ => return None,
            };
            Instruction::Register {
                operation,
                destination,
                first,
                second,
            }
        }
        0x0f if function_code == 0 => Instruction::Fence,
        0x73 if function_code == 0 && destination == 0 && first == 0 => {
            match word >> 20 {
                0 => Instruction::EnvironmentCall,
                1 => Instruction::Breakpoint,
                _ => return None,
            }
        }
        _ => return None,
    };
    Some(instruction)
}

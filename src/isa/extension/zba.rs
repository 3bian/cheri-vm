// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::fields::Fields;
use crate::isa::instruction::{
    ImmediateOperation, Instruction, RegisterOperation,
};

pub fn standard(word: u32, xlen: u32) -> Option<Instruction> {
    use RegisterOperation::*;
    let fields = Fields::of(word, xlen);
    Some(match (fields.opcode, fields.prefix, fields.function_code) {
        (0x33, 0x10, 2) => fields.register(ShiftLeft1Add),
        (0x33, 0x10, 4) => fields.register(ShiftLeft2Add),
        (0x33, 0x10, 6) => fields.register(ShiftLeft3Add),
        (0x3b, 0x04, 0) if fields.wide => fields.register(AddUnsignedWord),
        (0x3b, 0x10, 2) if fields.wide => {
            fields.register(ShiftLeft1AddUnsignedWord)
        }
        (0x3b, 0x10, 4) if fields.wide => {
            fields.register(ShiftLeft2AddUnsignedWord)
        }
        (0x3b, 0x10, 6) if fields.wide => {
            fields.register(ShiftLeft3AddUnsignedWord)
        }
        (0x1b, 0x04 | 0x05, 1) if fields.wide => fields.immediate(
            ImmediateOperation::ShiftLeftUnsignedWord,
            ((word >> 20) & 63) as i32,
        ),
        _ => return None,
    })
}

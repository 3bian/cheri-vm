// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::fields::Fields;
use crate::isa::instruction::{
    ImmediateOperation, Instruction, RegisterOperation,
};

pub(super) fn rotations(word: u32, xlen: u32) -> Option<Instruction> {
    use RegisterOperation::*;
    let fields = Fields::of(word, xlen);
    let reverse = if fields.wide { 0x6b8 } else { 0x698 };
    Some(match (fields.opcode, fields.prefix, fields.function_code) {
        (0x33, 0x20, 7) => fields.register(AndNot),
        (0x33, 0x20, 6) => fields.register(OrNot),
        (0x33, 0x20, 4) => fields.register(XorNot),
        (0x33, 0x30, 1) => fields.register(RotateLeft),
        (0x33, 0x30, 5) => fields.register(RotateRight),
        (0x3b, 0x30, 1) if fields.wide => fields.register_word(RotateLeft),
        (0x3b, 0x30, 5) if fields.wide => fields.register_word(RotateRight),
        (0x1b, 0x30, 5) if fields.wide => fields.immediate_word(
            ImmediateOperation::RotateRight,
            fields.second.into(),
        ),
        (0x13, _, 5) if fields.selector == reverse => {
            fields.immediate(ImmediateOperation::ByteReverse, 0)
        }
        (0x13, _, 5) => match fields.shift()? {
            (0x18, amount) => {
                fields.immediate(ImmediateOperation::RotateRight, amount)
            }
            _ => return None,
        },
        _ => return None,
    })
}

pub fn standard(word: u32, xlen: u32) -> Option<Instruction> {
    use ImmediateOperation::*;
    use RegisterOperation::{Max, MaxUnsigned, Min, MinUnsigned, Pack};
    if let Some(instruction) = rotations(word, xlen) {
        return Some(instruction);
    }
    let fields = Fields::of(word, xlen);
    let count = |selector| match selector {
        0x600 => Some(CountLeadingZeros),
        0x601 => Some(CountTrailingZeros),
        0x602 => Some(CountPopulation),
        _ => None,
    };
    Some(match (fields.opcode, fields.function_code) {
        (0x13, 1) => fields.immediate(
            match fields.selector {
                0x604 => SignExtendByte,
                0x605 => SignExtendHalfword,
                selector => count(selector)?,
            },
            0,
        ),
        (0x1b, 1) if fields.wide => {
            fields.immediate_word(count(fields.selector)?, 0)
        }
        (0x13, 5) if fields.selector == 0x287 => {
            fields.immediate(OrCombineBytes, 0)
        }
        (0x33, 4..=7) if fields.prefix == 5 => {
            fields.register(match fields.function_code {
                4 => Min,
                5 => MinUnsigned,
                6 => Max,
                _ => MaxUnsigned,
            })
        }
        (0x33, 4) if !fields.wide && fields.selector == 0x080 => {
            fields.register(Pack)
        }
        (0x3b, 4) if fields.wide && fields.selector == 0x080 => {
            fields.register_word(Pack)
        }
        _ => return None,
    })
}

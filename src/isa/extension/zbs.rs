// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::fields::Fields;
use crate::isa::instruction::{
    ImmediateOperation, Instruction, RegisterOperation,
};

pub fn standard(word: u32, xlen: u32) -> Option<Instruction> {
    let fields = Fields::of(word, xlen);
    match (fields.opcode, fields.function_code) {
        (0x33, 1 | 5) => {
            use RegisterOperation::*;
            Some(
                fields.register(match (fields.prefix, fields.function_code) {
                    (0x24, 1) => BitClear,
                    (0x24, 5) => BitExtract,
                    (0x34, 1) => BitInvert,
                    (0x14, 1) => BitSet,
                    _ => return None,
                }),
            )
        }
        (0x13, 1 | 5) => {
            use ImmediateOperation::*;
            let (prefix, amount) = fields.shift()?;
            Some(fields.immediate(
                match (prefix, fields.function_code) {
                    (0x12, 1) => BitClear,
                    (0x12, 5) => BitExtract,
                    (0x1a, 1) => BitInvert,
                    (0x0a, 1) => BitSet,
                    _ => return None,
                },
                amount,
            ))
        }
        _ => None,
    }
}

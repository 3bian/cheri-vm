// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::fields::Fields;
use crate::isa::instruction::{Instruction, RegisterOperation};

pub(super) fn multiplies(word: u32, xlen: u32) -> Option<Instruction> {
    let fields = Fields::of(word, xlen);
    match (fields.opcode, fields.prefix, fields.function_code) {
        (0x33, 5, 1) => Some(fields.register(RegisterOperation::CarrylessMul)),
        (0x33, 5, 3) => {
            Some(fields.register(RegisterOperation::CarrylessMulHigh))
        }
        _ => None,
    }
}

pub fn standard(word: u32, xlen: u32) -> Option<Instruction> {
    let fields = Fields::of(word, xlen);
    match (fields.opcode, fields.prefix, fields.function_code) {
        (0x33, 5, 2) => {
            Some(fields.register(RegisterOperation::CarrylessMulReversed))
        }
        _ => multiplies(word, xlen),
    }
}

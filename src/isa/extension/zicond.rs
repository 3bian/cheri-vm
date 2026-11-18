// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::fields::Fields;
use crate::isa::instruction::{Instruction, RegisterOperation};

pub fn standard(word: u32, xlen: u32) -> Option<Instruction> {
    let fields = Fields::of(word, xlen);
    match (fields.opcode, fields.prefix, fields.function_code) {
        (0x33, 7, 5) => Some(fields.register(RegisterOperation::ZeroIfZero)),
        (0x33, 7, 7) => Some(fields.register(RegisterOperation::ZeroIfNonzero)),
        _ => None,
    }
}

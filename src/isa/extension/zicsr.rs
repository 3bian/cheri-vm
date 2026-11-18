// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::instruction::{CsrOperation, Instruction};

pub fn standard(word: u32, _: u32) -> Option<Instruction> {
    if word & 0x7f != 0x73 {
        return None;
    }
    let destination = ((word >> 7) & 31) as u8;
    let function_code = (word >> 12) & 7;
    let first = ((word >> 15) & 31) as u8;
    match function_code {
        0 => super::privileged::machine(word),
        1..=3 | 5..=7 => Some(Instruction::Csr {
            operation: match function_code % 4 {
                1 => CsrOperation::ReadWrite,
                2 => CsrOperation::ReadSet,
                _ => CsrOperation::ReadClear,
            },
            destination,
            source: first,
            address: (word >> 20) as u16,
            immediate: function_code >= 5,
        }),
        _ => None,
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::instruction::{Instruction, RegisterOperation};

pub fn standard(word: u32, xlen: u32) -> Option<Instruction> {
    use RegisterOperation::*;
    if word >> 25 != 1 {
        return None;
    }
    let function_code = (word >> 12) & 7;
    let destination = ((word >> 7) & 31) as u8;
    let first = ((word >> 15) & 31) as u8;
    let second = ((word >> 20) & 31) as u8;
    match word & 0x7f {
        0x33 => Some(Instruction::Register {
            operation: match function_code {
                0 => Mul,
                1 => MulHighSigned,
                2 => MulHighSignedUnsigned,
                3 => MulHighUnsigned,
                4 => DivSigned,
                5 => DivUnsigned,
                6 => RemainderSigned,
                _ => RemainderUnsigned,
            },
            destination,
            first,
            second,
        }),
        0x3b if xlen == 64 => Some(Instruction::RegisterWord {
            operation: match function_code {
                0 => Mul,
                4 => DivSigned,
                5 => DivUnsigned,
                6 => RemainderSigned,
                7 => RemainderUnsigned,
                _ => return None,
            },
            destination,
            first,
            second,
        }),
        _ => None,
    }
}

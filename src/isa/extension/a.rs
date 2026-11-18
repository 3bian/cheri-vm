// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::instruction::{AtomicOperation, AtomicWidth, Instruction};

pub fn standard(word: u32, xlen: u32) -> Option<Instruction> {
    use AtomicOperation::*;
    if word & 0x7f != 0x2f {
        return None;
    }
    let width = match (word >> 12) & 7 {
        2 => AtomicWidth::Word,
        3 if xlen == 64 => AtomicWidth::Doubleword,
        _ => return None,
    };
    let source = ((word >> 20) & 31) as u8;
    let operation = match word >> 27 {
        0 => Add,
        1 => Swap,
        2 if source == 0 => LoadReserved,
        3 => StoreConditional,
        4 => Xor,
        8 => Or,
        12 => And,
        16 => Minimum,
        20 => Maximum,
        24 => MinimumUnsigned,
        28 => MaximumUnsigned,
        _ => return None,
    };
    Some(Instruction::Atomic {
        operation,
        width,
        destination: ((word >> 7) & 31) as u8,
        address: ((word >> 15) & 31) as u8,
        source,
    })
}

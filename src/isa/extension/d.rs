// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::f::decoded;
use crate::isa::instruction::{FloatOperation, FloatWidth, Instruction};

pub fn compressed(value: u16, _: u32) -> Option<Instruction> {
    let value = u32::from(value);
    let field = |shift: u32, mask: u32| (value >> shift) & mask;
    let near = |shift: u32| 8 + field(shift, 7) as u8;
    let width = FloatWidth::Double;
    let offset = (field(10, 7) << 3) | (field(5, 3) << 6);
    Some(match (value & 3, value >> 13) {
        (0, 1) => Instruction::FloatLoad {
            width,
            destination: near(2),
            base: near(7),
            offset: offset as i32,
        },
        (0, 5) => Instruction::FloatStore {
            width,
            source: near(2),
            base: near(7),
            offset: offset as i32,
        },
        (2, 1) => Instruction::FloatLoad {
            width,
            destination: field(7, 31) as u8,
            base: 2,
            offset: ((field(12, 1) << 5)
                | (field(5, 3) << 3)
                | (field(2, 7) << 6)) as i32,
        },
        (2, 5) => Instruction::FloatStore {
            width,
            source: field(2, 31) as u8,
            base: 2,
            offset: ((field(10, 7) << 3) | (field(7, 7) << 6)) as i32,
        },
        _ => return None,
    })
}

pub fn standard(word: u32, xlen: u32) -> Option<Instruction> {
    let resized = |width| {
        let rounding = ((word >> 12) & 7) as u8;
        (!matches!(rounding, 5 | 6)).then_some(Instruction::Float {
            operation: FloatOperation::Resize,
            width,
            destination: ((word >> 7) & 31) as u8,
            first: ((word >> 15) & 31) as u8,
            second: 0,
            third: 0,
            rounding,
        })
    };
    match (word & 0x7f, word >> 20) {
        (0x53, 0x401) => resized(FloatWidth::Single),
        (0x53, 0x420) => resized(FloatWidth::Double),
        _ => decoded(word, xlen, FloatWidth::Double),
    }
}

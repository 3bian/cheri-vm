// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::decode::sign_extend;
use crate::isa::instruction::{FloatOperation, FloatWidth, Instruction};

pub fn compressed(value: u16, xlen: u32) -> Option<Instruction> {
    let value = u32::from(value);
    let field = |shift: u32, mask: u32| (value >> shift) & mask;
    let near = |shift: u32| 8 + field(shift, 7) as u8;
    if xlen != 32 {
        return None;
    }
    let width = FloatWidth::Single;
    let offset = (field(10, 7) << 3) | (field(6, 1) << 2) | (field(5, 1) << 6);
    Some(match (value & 3, value >> 13) {
        (0, 3) => Instruction::FloatLoad {
            width,
            destination: near(2),
            base: near(7),
            offset: offset as i32,
        },
        (0, 7) => Instruction::FloatStore {
            width,
            source: near(2),
            base: near(7),
            offset: offset as i32,
        },
        (2, 3) => Instruction::FloatLoad {
            width,
            destination: field(7, 31) as u8,
            base: 2,
            offset: ((field(12, 1) << 5)
                | (field(4, 7) << 2)
                | (field(2, 3) << 6)) as i32,
        },
        (2, 7) => Instruction::FloatStore {
            width,
            source: field(2, 31) as u8,
            base: 2,
            offset: ((field(9, 15) << 2) | (field(7, 3) << 6)) as i32,
        },
        _ => return None,
    })
}

pub(super) fn decoded(
    word: u32,
    xlen: u32,
    width: FloatWidth,
) -> Option<Instruction> {
    use FloatOperation::*;
    let destination = ((word >> 7) & 31) as u8;
    let function = (word >> 12) & 7;
    let first = ((word >> 15) & 31) as u8;
    let second = ((word >> 20) & 31) as u8;
    let third = (word >> 27) as u8;
    let format = (word >> 25) & 3;
    let (size, code) = match width {
        FloatWidth::Double => (3, 1),
        FloatWidth::Single => (2, 0),
    };
    let wide = xlen == 64;
    let whole = wide || width == FloatWidth::Single;
    let operation = match word & 0x7f {
        0x07 if function == size => {
            return Some(Instruction::FloatLoad {
                width,
                destination,
                base: first,
                offset: sign_extend(word >> 20, 12),
            });
        }
        0x27 if function == size => {
            return Some(Instruction::FloatStore {
                width,
                source: second,
                base: first,
                offset: sign_extend(
                    ((word >> 25) << 5) | ((word >> 7) & 31),
                    12,
                ),
            });
        }
        0x43 if format == code => MultiplyAdd,
        0x47 if format == code => MultiplySubtract,
        0x4b if format == code => NegatedMultiplySubtract,
        0x4f if format == code => NegatedMultiplyAdd,
        0x53 if format == code => match (word >> 27, function, second) {
            (0x00, ..) => Add,
            (0x01, ..) => Subtract,
            (0x02, ..) => Multiply,
            (0x03, ..) => Divide,
            (0x0b, _, 0) => SquareRoot,
            (0x04, 0, _) => SignInject,
            (0x04, 1, _) => SignInjectNegated,
            (0x04, 2, _) => SignInjectXor,
            (0x05, 0, _) => Minimum,
            (0x05, 1, _) => Maximum,
            (0x14, 0, _) => LessOrEqual,
            (0x14, 1, _) => LessThan,
            (0x14, 2, _) => Equal,
            (0x18, _, 0) => ToWord,
            (0x18, _, 1) => ToUnsignedWord,
            (0x18, _, 2) if wide => ToLong,
            (0x18, _, 3) if wide => ToUnsignedLong,
            (0x1a, _, 0) => FromWord,
            (0x1a, _, 1) => FromUnsignedWord,
            (0x1a, _, 2) if wide => FromLong,
            (0x1a, _, 3) if wide => FromUnsignedLong,
            (0x1c, 0, 0) if whole => MoveToInteger,
            (0x1c, 1, 0) => Classify,
            (0x1e, 0, 0) if whole => MoveFromInteger,
            _ => return None,
        },
        _ => return None,
    };
    if operation.rounds() && matches!(function, 5 | 6) {
        return None;
    }
    Some(Instruction::Float {
        operation,
        width,
        destination,
        first,
        second,
        third: if word & 0x7f == 0x53 { 0 } else { third },
        rounding: function as u8,
    })
}

pub fn standard(word: u32, xlen: u32) -> Option<Instruction> {
    decoded(word, xlen, FloatWidth::Single)
}

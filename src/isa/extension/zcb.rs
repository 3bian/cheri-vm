// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::instruction::{
    ImmediateOperation, Instruction, LoadKind, RegisterOperation, StoreKind,
};

fn arithmetic(value: u16, xlen: u32) -> Option<Instruction> {
    let destination = register(value, 7);
    let immediate = |operation, immediate| Instruction::Immediate {
        operation,
        destination,
        source: destination,
        immediate,
    };
    let pack = if xlen == 64 {
        Instruction::RegisterWord {
            operation: RegisterOperation::Pack,
            destination,
            first: destination,
            second: 0,
        }
    } else {
        Instruction::Register {
            operation: RegisterOperation::Pack,
            destination,
            first: destination,
            second: 0,
        }
    };
    Some(match ((value >> 5) & 3, (value >> 2) & 7) {
        (2, _) => Instruction::Register {
            operation: RegisterOperation::Mul,
            destination,
            first: destination,
            second: register(value, 2),
        },
        (3, 0) => immediate(ImmediateOperation::And, 0xff),
        (3, 1) => immediate(ImmediateOperation::SignExtendByte, 0),
        (3, 2) => pack,
        (3, 3) => immediate(ImmediateOperation::SignExtendHalfword, 0),
        (3, 4) if xlen == 64 => Instruction::Register {
            operation: RegisterOperation::AddUnsignedWord,
            destination,
            first: destination,
            second: 0,
        },
        (3, 5) => immediate(ImmediateOperation::Xor, -1),
        _ => return None,
    })
}

pub fn compressed(value: u16, xlen: u32) -> Option<Instruction> {
    match (value & 3, value >> 10) {
        (0, 0x20..=0x23) => memory(value),
        (1, 0x27) => arithmetic(value, xlen),
        _ => None,
    }
}

fn memory(value: u16) -> Option<Instruction> {
    let base = register(value, 7);
    let other = register(value, 2);
    let low = i32::from((value >> 6) & 1);
    let high = i32::from((value >> 5) & 1) << 1;
    let load = |kind, offset| Instruction::Load {
        kind,
        destination: other,
        base,
        offset,
    };
    let store = |kind, offset| Instruction::Store {
        kind,
        source: other,
        base,
        offset,
    };
    Some(match ((value >> 10) & 3, low) {
        (0, _) => load(LoadKind::UnsignedByte, high | low),
        (1, 0) => load(LoadKind::UnsignedHalfword, high),
        (1, _) => load(LoadKind::SignedHalfword, high),
        (2, _) => store(StoreKind::Byte, high | low),
        (3, 0) => store(StoreKind::Halfword, high),
        _ => return None,
    })
}

fn register(value: u16, shift: u32) -> u8 {
    ((value >> shift) & 7) as u8 + 8
}

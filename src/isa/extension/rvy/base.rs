// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Operation;
use super::encoding::*;
use crate::isa::decode::sign_extend;
use crate::isa::extension::Decoding;
use crate::isa::instruction::{
    AtomicOperation, AtomicWidth, Instruction, LoadKind, StoreKind,
};

fn bounds_length(immediate: u32) -> i16 {
    (match immediate {
        0 => 4096,
        1..=255 => immediate,
        256..=287 => 256 | (immediate & 15) << 4 | (immediate >> 4 & 1) << 3,
        _ => (immediate & 0xff) << 4,
    }) as i16
}

pub fn compressed(value: u16, _: u32) -> Decoding {
    let field = |shift: u32, mask: u16| i32::from((value >> shift) & mask);
    let register = |shift: u32| ((value >> shift) & 31) as u8;
    let prime = |shift: u32| ((value >> shift) & 7) as u8 + 8;
    let add = |destination, immediate: i32| Instruction::Rvy {
        operation: Operation::AddImmediate,
        destination,
        first: 2,
        second: 0,
        immediate: immediate as i16,
    };
    let near = field(12, 1) << 5
        | field(11, 1) << 4
        | field(10, 1) << 8
        | field(5, 3) << 6;
    let function_code = value >> 13;
    let unchanged = field(12, 1) == 0 && field(2, 31) == 0;
    let reserved = match (value & 3, function_code) {
        (1 | 2, 0) => register(7) != 0 && unchanged,
        (1, 4) => field(11, 1) == 0 && unchanged,
        (2, COMPRESSED_LOAD) => register(7) == 0,
        _ => false,
    };
    if reserved {
        return Decoding::Reserved;
    }
    let found = match (value & 3, function_code) {
        (0, 0) => {
            let immediate = field(7, 15) << 6
                | field(11, 3) << 4
                | field(5, 1) << 3
                | field(6, 1) << 2;
            if immediate == 0 {
                return Decoding::Absent;
            }
            add(prime(2), immediate)
        }
        (0, COMPRESSED_LOAD) => Instruction::Load {
            kind: LoadKind::Capability,
            destination: prime(2),
            base: prime(7),
            offset: near,
        },
        (0, COMPRESSED_STORE) => Instruction::Store {
            kind: StoreKind::Capability,
            source: prime(2),
            base: prime(7),
            offset: near,
        },
        (1, 3) if register(7) == 2 => {
            let immediate = field(12, 1) << 9
                | field(3, 3) << 7
                | field(5, 1) << 6
                | field(2, 1) << 5
                | field(6, 1) << 4;
            if immediate == 0 {
                return Decoding::Absent;
            }
            add(2, sign_extend(immediate as u32, 10))
        }
        (2, COMPRESSED_LOAD) => Instruction::Load {
            kind: LoadKind::Capability,
            destination: register(7),
            base: 2,
            offset: field(12, 1) << 5 | field(6, 1) << 4 | field(2, 15) << 6,
        },
        (2, COMPRESSED_STORE) => Instruction::Store {
            kind: StoreKind::Capability,
            source: register(2),
            base: 2,
            offset: field(11, 3) << 4 | field(7, 15) << 6,
        },
        (2, 4) if field(12, 1) == 0 && register(7) != 0 && register(2) != 0 => {
            Instruction::Rvy {
                operation: Operation::Move,
                destination: register(7),
                first: register(2),
                second: 0,
                immediate: 0,
            }
        }
        _ => return Decoding::Absent,
    };
    Decoding::Found(found)
}

fn register(
    prefix: u32,
    first: u8,
    second: u8,
) -> Option<(Operation, u8, u8, i16)> {
    let listed = |table: &[(u32, Operation)], code: u32| {
        table
            .iter()
            .find(|(listed, _)| *listed == code)
            .map(|(_, operation)| *operation)
    };
    if prefix == QUERIES {
        return Some((listed(&QUERY, second.into())?, first, 0, 0));
    }
    if prefix == MASKS {
        return (second == 0).then_some((
            Operation::AlignmentMask,
            first,
            0,
            0,
        ));
    }
    if let Some((_, operation, amount)) =
        SHIFT_ADD.iter().find(|(code, _, _)| *code == prefix)
    {
        return Some((*operation, first, second, *amount));
    }
    Some(match listed(&TWO_REGISTER, prefix)? {
        Operation::Add if second == 0 => (Operation::Move, first, 0, 0),
        Operation::SealEntry if first != 0 => return None,
        Operation::SealEntry => (Operation::SealEntry, second, 0, 0),
        operation => (operation, first, second, 0),
    })
}

pub fn standard(word: u32, _: u32) -> Option<Instruction> {
    if word & 0x7f != MAJOR {
        return None;
    }
    let destination = ((word >> 7) & 31) as u8;
    let first = ((word >> 15) & 31) as u8;
    let second = ((word >> 20) & 31) as u8;
    let prefix = word >> 25;
    let rvy = |operation, first, second, immediate| Instruction::Rvy {
        operation,
        destination,
        first,
        second,
        immediate,
    };
    Some(match (word >> 12) & 7 {
        REGISTER => register(prefix, first, second).map(
            |(operation, first, second, immediate)| {
                rvy(operation, first, second, immediate)
            },
        )?,
        LOAD if first != 0 => Instruction::Load {
            kind: LoadKind::Capability,
            destination,
            base: first,
            offset: sign_extend(word >> 20, 12),
        },
        STORE if first != 0 => Instruction::Store {
            kind: StoreKind::Capability,
            source: second,
            base: first,
            offset: sign_extend((prefix << 5) | u32::from(destination), 12),
        },
        ATOMIC if first != 0 => Instruction::Atomic {
            operation: match (word >> 27, second) {
                (ATOMIC_LOAD_RESERVED, 0) => AtomicOperation::LoadReserved,
                (ATOMIC_STORE_CONDITIONAL, _) => {
                    AtomicOperation::StoreConditional
                }
                (ATOMIC_SWAP, _) => AtomicOperation::Swap,
                _ => return None,
            },
            width: AtomicWidth::Capability,
            destination,
            address: first,
            source: second,
        },
        ADD_IMMEDIATE => rvy(
            Operation::AddImmediate,
            first,
            0,
            sign_extend(word >> 20, 12) as i16,
        ),
        MISCELLANEOUS if word >> 29 == SET_BOUNDS_IMMEDIATE => rvy(
            Operation::SetBoundsImmediate,
            first,
            0,
            bounds_length((word >> 20) & 0x1ff),
        ),
        _ => return None,
    })
}

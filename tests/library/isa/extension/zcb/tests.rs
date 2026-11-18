// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::decode::compressed;
use crate::isa::extension::ExtensionSet;
use crate::isa::instruction::{
    ImmediateOperation, Instruction, LoadKind, RegisterOperation, StoreKind,
};

const ALL: [&str; 6] = ["i", "m", "c", "zba", "zbb", "zcb"];

fn set(names: &[&str]) -> ExtensionSet {
    ExtensionSet::named(names.iter().copied()).unwrap()
}

fn short(xlen: u32, halfword: u16) -> Option<Instruction> {
    compressed(set(&ALL), xlen, halfword).map(|decoded| decoded.instruction)
}

#[test]
fn the_short_extensions_stand_for_the_long_instructions() {
    let unary = |operation, immediate| Instruction::Immediate {
        operation,
        destination: 9,
        source: 9,
        immediate,
    };
    let binary = |operation, second| Instruction::Register {
        operation,
        destination: 9,
        first: 9,
        second,
    };
    assert_eq!(
        short(64, 0x9ce1),
        Some(unary(ImmediateOperation::And, 0xff))
    );
    assert_eq!(
        short(64, 0x9ce5),
        Some(unary(ImmediateOperation::SignExtendByte, 0))
    );
    assert_eq!(
        short(64, 0x9ce9),
        Some(Instruction::RegisterWord {
            operation: RegisterOperation::Pack,
            destination: 9,
            first: 9,
            second: 0,
        })
    );
    assert_eq!(short(32, 0x9ce9), Some(binary(RegisterOperation::Pack, 0)));
    assert_eq!(
        short(64, 0x9ced),
        Some(unary(ImmediateOperation::SignExtendHalfword, 0))
    );
    assert_eq!(
        short(64, 0x9cf1),
        Some(binary(RegisterOperation::AddUnsignedWord, 0))
    );
    assert_eq!(short(32, 0x9cf1), None);
    assert_eq!(short(64, 0x9cf5), Some(unary(ImmediateOperation::Xor, -1)));
    assert_eq!(short(64, 0x9cc9), Some(binary(RegisterOperation::Mul, 10)));
    assert_eq!(short(64, 0x9cf9), None);
}

#[test]
fn the_short_loads_and_stores_carry_a_two_bit_offset() {
    let load = |kind, offset| Instruction::Load {
        kind,
        destination: 9,
        base: 10,
        offset,
    };
    let store = |kind, offset| Instruction::Store {
        kind,
        source: 9,
        base: 10,
        offset,
    };
    assert_eq!(short(64, 0x8164), Some(load(LoadKind::UnsignedByte, 3)));
    assert_eq!(short(64, 0x8524), Some(load(LoadKind::UnsignedHalfword, 2)));
    assert_eq!(short(64, 0x8564), Some(load(LoadKind::SignedHalfword, 2)));
    assert_eq!(short(64, 0x8944), Some(store(StoreKind::Byte, 1)));
    assert_eq!(short(32, 0x8d24), Some(store(StoreKind::Halfword, 2)));
    assert_eq!(short(64, 0x8d64), None);
}

#[test]
fn without_the_extension_the_short_forms_are_not_instructions() {
    for halfword in [0x8164, 0x8d24, 0x9ce1, 0x9cc9] {
        assert!(
            compressed(set(&["i", "m", "c", "zba", "zbb"]), 64, halfword)
                .is_none()
        );
    }
}

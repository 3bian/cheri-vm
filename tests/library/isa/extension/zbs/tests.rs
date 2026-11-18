// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::decode::standard;
use crate::isa::extension::ExtensionSet;
use crate::isa::instruction::{
    ImmediateOperation, Instruction, RegisterOperation,
};

fn check(name: &str, xlen: u32, rows: &[(u32, Instruction)]) {
    for (word, instruction) in rows {
        assert_eq!(
            decoded(&["i", name], xlen, *word),
            Some(*instruction),
            "{word:#x}"
        );
        assert_eq!(decoded(&["i"], xlen, *word), None, "{word:#x}");
    }
}

fn decoded(names: &[&str], xlen: u32, word: u32) -> Option<Instruction> {
    standard(set(names), xlen, word).map(|decoded| decoded.instruction)
}

fn immediate(operation: ImmediateOperation, immediate: i32) -> Instruction {
    Instruction::Immediate {
        operation,
        destination: 1,
        source: 2,
        immediate,
    }
}

fn register(operation: RegisterOperation, second: u8) -> Instruction {
    Instruction::Register {
        operation,
        destination: 1,
        first: 2,
        second,
    }
}

fn set(names: &[&str]) -> ExtensionSet {
    ExtensionSet::named(names.iter().copied()).unwrap()
}

#[test]
fn the_single_bit_instructions_decode_with_a_shift_as_wide_as_the_machine() {
    use RegisterOperation::*;
    check(
        "zbs",
        64,
        &[
            (0x4831_10b3, register(BitClear, 3)),
            (0x4ad1_1093, immediate(ImmediateOperation::BitClear, 45)),
            (0x4831_50b3, register(BitExtract, 3)),
            (0x4ad1_5093, immediate(ImmediateOperation::BitExtract, 45)),
            (0x6831_10b3, register(BitInvert, 3)),
            (0x6ad1_1093, immediate(ImmediateOperation::BitInvert, 45)),
            (0x2831_10b3, register(BitSet, 3)),
            (0x2ad1_1093, immediate(ImmediateOperation::BitSet, 45)),
        ],
    );
    check(
        "zbs",
        32,
        &[(0x48d1_5093, immediate(ImmediateOperation::BitExtract, 13))],
    );
    assert_eq!(decoded(&["i", "zbs"], 32, 0x4ad1_5093), None);
}

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
fn the_address_instructions_decode_and_the_word_forms_need_64_bits() {
    use RegisterOperation::*;
    check(
        "zba",
        64,
        &[
            (0x2031_20b3, register(ShiftLeft1Add, 3)),
            (0x2031_40b3, register(ShiftLeft2Add, 3)),
            (0x2031_60b3, register(ShiftLeft3Add, 3)),
            (0x0831_00bb, register(AddUnsignedWord, 3)),
            (0x2031_20bb, register(ShiftLeft1AddUnsignedWord, 3)),
            (0x2031_40bb, register(ShiftLeft2AddUnsignedWord, 3)),
            (0x2031_60bb, register(ShiftLeft3AddUnsignedWord, 3)),
            (
                0x0a31_109b,
                immediate(ImmediateOperation::ShiftLeftUnsignedWord, 35),
            ),
        ],
    );
    check("zba", 32, &[(0x2031_20b3, register(ShiftLeft1Add, 3))]);
    for word in [0x0831_00bb, 0x2031_20bb, 0x0a31_109b] {
        assert_eq!(decoded(&["i", "zba"], 32, word), None);
    }
}

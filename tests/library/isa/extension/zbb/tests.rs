// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::decode::standard;
use crate::isa::extension::ExtensionSet;
use crate::isa::instruction::{
    ImmediateOperation, Instruction, RegisterOperation,
};

#[test]
fn byte_reversal_and_halfword_extension_are_encoded_by_width() {
    use ImmediateOperation::*;
    check(
        "zbb",
        32,
        &[
            (0x60d1_5093, immediate(RotateRight, 13)),
            (0x6981_5093, immediate(ByteReverse, 0)),
            (0x0801_40b3, register(RegisterOperation::Pack, 0)),
        ],
    );
    assert_eq!(decoded(&["i", "zbb"], 32, 0x6b81_5093), None);
    assert_eq!(decoded(&["i", "zbb"], 64, 0x6981_5093), None);
    assert_eq!(decoded(&["i", "zbb"], 32, 0x62d1_5093), None);
    assert_eq!(decoded(&["i", "zbb"], 32, 0x0831_40b3), None);
    assert_eq!(decoded(&["i", "zbb"], 64, 0x0801_40b3), None);
}

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

fn immediate_word(
    operation: ImmediateOperation,
    immediate: i32,
) -> Instruction {
    Instruction::ImmediateWord {
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

fn register_word(operation: RegisterOperation, second: u8) -> Instruction {
    Instruction::RegisterWord {
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
fn the_bit_instructions_decode_at_64_bits() {
    use ImmediateOperation::*;
    use RegisterOperation::{
        AndNot, Max, MaxUnsigned, Min, MinUnsigned, OrNot, Pack, RotateLeft,
        XorNot,
    };
    check(
        "zbb",
        64,
        &[
            (0x4031_70b3, register(AndNot, 3)),
            (0x4031_60b3, register(OrNot, 3)),
            (0x4031_40b3, register(XorNot, 3)),
            (0x6001_1093, immediate(CountLeadingZeros, 0)),
            (0x6011_1093, immediate(CountTrailingZeros, 0)),
            (0x6021_1093, immediate(CountPopulation, 0)),
            (0x6001_109b, immediate_word(CountLeadingZeros, 0)),
            (0x6011_109b, immediate_word(CountTrailingZeros, 0)),
            (0x6021_109b, immediate_word(CountPopulation, 0)),
            (0x6041_1093, immediate(SignExtendByte, 0)),
            (0x6051_1093, immediate(SignExtendHalfword, 0)),
            (0x0801_40bb, register_word(Pack, 0)),
            (0x0a31_60b3, register(Max, 3)),
            (0x0a31_70b3, register(MaxUnsigned, 3)),
            (0x0a31_40b3, register(Min, 3)),
            (0x0a31_50b3, register(MinUnsigned, 3)),
            (0x6031_10b3, register(RotateLeft, 3)),
            (0x6031_50b3, register(RegisterOperation::RotateRight, 3)),
            (0x62d1_5093, immediate(RotateRight, 45)),
            (0x6031_10bb, register_word(RotateLeft, 3)),
            (
                0x6031_50bb,
                register_word(RegisterOperation::RotateRight, 3),
            ),
            (0x60d1_509b, immediate_word(RotateRight, 13)),
            (0x2871_5093, immediate(OrCombineBytes, 0)),
            (0x6b81_5093, immediate(ByteReverse, 0)),
        ],
    );
}

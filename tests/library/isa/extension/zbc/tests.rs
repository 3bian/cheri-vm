// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::decode::standard;
use crate::isa::extension::ExtensionSet;
use crate::isa::instruction::{Instruction, RegisterOperation};

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
fn the_three_carry_less_multiplies_decode() {
    use RegisterOperation::*;
    for xlen in [32, 64] {
        check(
            "zbc",
            xlen,
            &[
                (0x0a31_10b3, register(CarrylessMul, 3)),
                (0x0a31_30b3, register(CarrylessMulHigh, 3)),
                (0x0a31_20b3, register(CarrylessMulReversed, 3)),
            ],
        );
    }
}

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
fn the_two_conditional_zeroes_decode() {
    use RegisterOperation::*;
    for xlen in [32, 64] {
        check(
            "zicond",
            xlen,
            &[
                (0x0e31_50b3, register(ZeroIfZero, 3)),
                (0x0e31_70b3, register(ZeroIfNonzero, 3)),
            ],
        );
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::extension::c::*;
use crate::isa::instruction::ImmediateOperation;
use crate::isa::instruction::Instruction;

#[test]
fn compressed_examples_decode_as_the_standard_says() {
    let nop = compressed(0x0001, 32).unwrap();
    assert!(matches!(nop, Instruction::Hint));
    assert_eq!(
        compressed(0x0085, 32).unwrap(),
        Instruction::Immediate {
            operation: ImmediateOperation::Add,
            destination: 1,
            source: 1,
            immediate: 1
        }
    );
    assert_eq!(
        compressed(0x8082, 32).unwrap(),
        Instruction::JumpAndLinkRegister {
            destination: 0,
            base: 1,
            offset: 0
        }
    );
    assert!(compressed(0, 32).is_none());
}

#[test]
fn the_64_bit_forms_replace_the_jump_and_link_and_add_the_doubleword_ones() {
    use crate::isa::instruction::{LoadKind, RegisterOperation, StoreKind};
    assert_eq!(
        compressed(0x347d, 64).unwrap(),
        Instruction::ImmediateWord {
            operation: ImmediateOperation::Add,
            destination: 8,
            source: 8,
            immediate: -1
        }
    );
    assert!(compressed(0x2001, 64).is_none());
    assert_eq!(
        compressed(0x6088, 64).unwrap(),
        Instruction::Load {
            kind: LoadKind::Doubleword,
            destination: 10,
            base: 9,
            offset: 0
        }
    );
    assert_eq!(
        compressed(0xe422, 64).unwrap(),
        Instruction::Store {
            kind: StoreKind::Doubleword,
            source: 8,
            base: 2,
            offset: 8
        }
    );
    assert_eq!(
        compressed(0x65a2, 64).unwrap(),
        Instruction::Load {
            kind: LoadKind::Doubleword,
            destination: 11,
            base: 2,
            offset: 8
        }
    );
    assert!(compressed(0x6002, 64).is_none());
    assert_eq!(
        compressed(0x9d01, 64).unwrap(),
        Instruction::RegisterWord {
            operation: RegisterOperation::Sub,
            destination: 10,
            first: 10,
            second: 8
        }
    );
    assert!(compressed(0x9d41, 64).is_none());
    assert!(compressed(0x9d01, 32).is_none());
    assert_eq!(
        compressed(0x9005, 64).unwrap(),
        Instruction::Immediate {
            operation: ImmediateOperation::ShiftRightLogical,
            destination: 8,
            source: 8,
            immediate: 33
        }
    );
    assert!(matches!(compressed(0x1002, 64).unwrap(), Instruction::Hint));
}

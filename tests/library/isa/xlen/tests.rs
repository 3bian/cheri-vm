// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::arithmetic::{
    perform_immediate_operation, perform_register_operation,
};
use crate::isa::instruction::{
    BranchCondition, ImmediateOperation, LoadKind, RegisterOperation,
};
use crate::isa::xlen::{Rv32, Rv64, Word, Xlen};

#[test]
fn a_branch_compares_whole_words_and_a_load_extends_to_the_word() {
    assert!(BranchCondition::LessThan.test(u64::MAX, 0));
    assert!(BranchCondition::GreaterOrEqualUnsigned.test(u64::MAX, 0));
    assert!(BranchCondition::LessThan.test(0x8000_0000_u32, 0));
    assert!(!BranchCondition::LessThan.test(0x8000_0000_u64, 0));
    assert_eq!(
        LoadKind::Word.extend::<u64>(0x8000_0000),
        0xffff_ffff_8000_0000
    );
    assert_eq!(LoadKind::Word.extend::<u32>(0x8000_0000), 0x8000_0000);
    assert_eq!(
        LoadKind::UnsignedWord.extend::<u64>(0x8000_0000),
        0x8000_0000
    );
    assert_eq!(
        LoadKind::SignedByte.extend::<u64>(0x80),
        0xffff_ffff_ffff_ff80
    );
    assert_eq!(LoadKind::SignedHalfword.extend::<u32>(0x8000), 0xffff_8000);
    assert_eq!(LoadKind::Doubleword.extend::<u64>(u64::MAX), u64::MAX);
    assert_eq!(
        (LoadKind::Doubleword.width(), LoadKind::UnsignedWord.width()),
        (8, 4)
    );
}

#[test]
fn a_word_converts_by_truncating_and_by_extending_its_sign() {
    assert_eq!((Rv32::BITS, Rv64::BITS), (32, 64));
    assert_eq!(u32::from_u64(0x1_2345_6789), 0x2345_6789);
    assert_eq!(u64::from_u64(0x1_2345_6789), 0x1_2345_6789);
    assert_eq!(u32::from_i32(-2), 0xffff_fffe);
    assert_eq!(u64::from_i32(-2), 0xffff_ffff_ffff_fffe);
    assert_eq!(0xffff_fffe_u32.to_i64(), -2);
    assert_eq!(0xffff_fffe_u32.to_u64(), 0xffff_fffe);
    assert_eq!(0xffff_fffe_u64.to_i64(), 0xffff_fffe);
    assert_eq!(u32::MAX.add_signed(2), 1);
    assert_eq!(u64::from(u32::MAX).add_signed(2), 0x1_0000_0001);
    assert_eq!(5_u64.add_signed(-6), u64::MAX);
}

#[test]
fn arithmetic_at_64_bits_shifts_by_six_bits_and_multiplies_into_128() {
    use RegisterOperation::*;
    let lowest = 1_u64 << 63;
    assert_eq!(perform_register_operation(ShiftLeft, 1_u64, 63), lowest);
    assert_eq!(perform_register_operation(ShiftLeft, 1_u32, 63), 1 << 31);
    assert_eq!(
        perform_register_operation(ShiftRightArithmetic, lowest, 63),
        u64::MAX
    );
    assert_eq!(
        perform_register_operation(MulHighUnsigned, u64::MAX, u64::MAX),
        u64::MAX - 1
    );
    assert_eq!(
        perform_register_operation(MulHighSigned, u64::MAX, u64::MAX),
        0
    );
    assert_eq!(
        perform_register_operation(MulHighSignedUnsigned, u64::MAX, u64::MAX),
        u64::MAX
    );
    assert_eq!(
        perform_register_operation(DivSigned, lowest, u64::MAX),
        lowest
    );
    assert_eq!(
        perform_register_operation(RemainderSigned, lowest, u64::MAX),
        0
    );
    assert_eq!(perform_register_operation(DivUnsigned, 7_u64, 0), u64::MAX);
    assert_eq!(perform_register_operation(RemainderUnsigned, 7_u64, 0), 7);
    assert_eq!(perform_register_operation(SetLessThan, u64::MAX, 0), 1);
    assert_eq!(
        perform_register_operation(SetLessThanUnsigned, u64::MAX, 0),
        0
    );
    assert_eq!(
        perform_immediate_operation(ImmediateOperation::Add, 0_u64, -1),
        u64::MAX
    );
    assert_eq!(
        perform_immediate_operation(
            ImmediateOperation::ShiftRightLogical,
            u64::MAX,
            60
        ),
        15
    );
    assert_eq!(
        perform_immediate_operation(ImmediateOperation::And, u64::MAX, -16),
        u64::MAX - 15
    );
}

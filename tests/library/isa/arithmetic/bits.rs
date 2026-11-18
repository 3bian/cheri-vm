// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::arithmetic::Arithmetic;
use crate::isa::instruction::{ImmediateOperation, RegisterOperation};

#[test]
fn a_carry_less_product_is_split_three_ways() {
    use RegisterOperation::*;
    assert_eq!(0b101_u32.register_operation(CarrylessMul, 0b11), 0b1111);
    assert_eq!(
        0x8000_0000_u32.register_operation(CarrylessMulHigh, 0b110),
        0b11
    );
    assert_eq!(
        0x8000_0000_u32.register_operation(CarrylessMulReversed, 0b110),
        0b110
    );
    let (first, second) =
        (0x8765_4321_0fed_cba9_u64, 0xdead_beef_0bad_f00d_u64);
    let product = (0..64)
        .filter(|bit| (second >> bit) & 1 != 0)
        .fold(0_u128, |sum, bit| sum ^ (u128::from(first) << bit));
    assert_eq!(
        first.register_operation(CarrylessMul, second),
        product as u64
    );
    assert_eq!(
        first.register_operation(CarrylessMulHigh, second),
        (product >> 64) as u64
    );
    assert_eq!(
        first.register_operation(CarrylessMulReversed, second),
        (product >> 63) as u64
    );
}

#[test]
fn a_permutation_looks_each_index_up_and_gives_zero_for_one_out_of_range() {
    use RegisterOperation::*;
    assert_eq!(
        0x4433_2211_u32.register_operation(Permute8, 0x0003_0104),
        0x1144_2200
    );
    assert_eq!(
        0x8877_6655_4433_2211_u64
            .register_operation(Permute8, 0x0007_0808_0000_0001),
        0x1188_0000_1111_1122
    );
    assert_eq!(
        0xfedc_ba98_u32.register_operation(Permute4, 0x0123_4567),
        0x89ab_cdef
    );
    assert_eq!(
        0xfedc_ba98_u32.register_operation(Permute4, 0x8000_0000),
        0x0888_8888
    );
}

#[test]
fn counting_extension_and_byte_order_work_at_both_widths() {
    use ImmediateOperation::*;
    assert_eq!(unary(CountLeadingZeros, 0_u32), 32);
    assert_eq!(unary(CountLeadingZeros, 1_u64), 63);
    assert_eq!(unary(CountTrailingZeros, 0_u64), 64);
    assert_eq!(unary(CountTrailingZeros, 0x80_u32), 7);
    assert_eq!(unary(CountPopulation, 0xf0f0_u32), 8);
    assert_eq!(unary(SignExtendByte, 0x1280_u32), 0xffff_ff80);
    assert_eq!(
        unary(SignExtendHalfword, 0x1_8000_u64),
        0xffff_ffff_ffff_8000
    );
    assert_eq!(unary(OrCombineBytes, 0x0100_0020_u32), 0xff00_00ff);
    assert_eq!(
        unary(OrCombineBytes, 0x0100_0000_0000_0300_u64),
        0xff00_0000_0000_ff00
    );
    assert_eq!(unary(ByteReverse, 0x1122_3344_u32), 0x4433_2211);
    assert_eq!(
        unary(ByteReverse, 0x1122_3344_5566_7788_u64),
        0x8877_6655_4433_2211
    );
    assert_eq!(unary(BitReverseBytes, 0x0180_f00f_u32), 0x8001_0ff0);
    assert_eq!(unary(Zip, 0xffff_0000_u32), 0xaaaa_aaaa);
    assert_eq!(unary(Unzip, 0xaaaa_aaaa_u32), 0xffff_0000);
    for value in [0x1234_5678_u32, 0xdead_beef, 1, 0x8000_0000] {
        assert_eq!(unary(Unzip, unary(Zip, value)), value);
    }
}

#[test]
fn the_shifted_forms_take_their_amount_from_the_immediate() {
    use ImmediateOperation::*;
    assert_eq!(
        0x8000_0001_u32.immediate_operation(RotateRight, 1),
        0xc000_0000
    );
    assert_eq!(1_u64.immediate_operation(RotateRight, 1), 1 << 63);
    assert_eq!(0_u64.immediate_operation(BitSet, 63), 1 << 63);
    assert_eq!(u32::MAX.immediate_operation(BitClear, 31), 0x7fff_ffff);
    assert_eq!(0_u32.immediate_operation(BitInvert, 4), 16);
    assert_eq!(0x10_u32.immediate_operation(BitExtract, 4), 1);
    assert_eq!(
        0xffff_ffff_8000_0001_u64.immediate_operation(ShiftLeftUnsignedWord, 4),
        0x8_0000_0010
    );
}

#[test]
fn two_register_bit_operations_work_at_both_widths() {
    use RegisterOperation::*;
    assert_eq!(0xff_u32.register_operation(AndNot, 0x0f), 0xf0);
    assert_eq!(0_u32.register_operation(OrNot, 0x0f), 0xffff_fff0);
    assert_eq!(0xff_u32.register_operation(XorNot, 0x0f), 0xffff_ff0f);
    assert_eq!(u32::MAX.register_operation(Max, 1), 1);
    assert_eq!(u32::MAX.register_operation(MaxUnsigned, 1), u32::MAX);
    assert_eq!(u64::MAX.register_operation(Min, 1), u64::MAX);
    assert_eq!(u64::MAX.register_operation(MinUnsigned, 1), 1);
    assert_eq!(0x8000_0001_u32.register_operation(RotateLeft, 33), 3);
    assert_eq!(3_u64.register_operation(RotateRight, 65), (1 << 63) | 1);
    assert_eq!(5_u32.register_operation(ShiftLeft1Add, 1), 11);
    assert_eq!(5_u32.register_operation(ShiftLeft2Add, 1), 21);
    assert_eq!(5_u64.register_operation(ShiftLeft3Add, 1), 41);
    assert_eq!(
        0xffff_ffff_0000_0005_u64.register_operation(AddUnsignedWord, 1),
        6
    );
    assert_eq!(
        0xffff_ffff_8000_0000_u64
            .register_operation(ShiftLeft3AddUnsignedWord, 1),
        0x4_0000_0001
    );
    assert_eq!(0_u64.register_operation(BitSet, 64 + 3), 8);
    assert_eq!(0xff_u32.register_operation(BitClear, 32), 0xfe);
    assert_eq!(0_u32.register_operation(BitInvert, 31), 1 << 31);
    assert_eq!(0x8000_0000_u32.register_operation(BitExtract, 31), 1);
    assert_eq!(
        0x1234_5678_u32.register_operation(Pack, 0xaaaa_bbbb),
        0xbbbb_5678
    );
    assert_eq!(
        0x1111_2222_3333_4444_u64
            .register_operation(Pack, 0x5555_6666_7777_8888),
        0x7777_8888_3333_4444
    );
    assert_eq!(0x1234_u32.register_operation(PackHigh, 0x5678), 0x7834);
    assert_eq!(7_u32.register_operation(ZeroIfZero, 0), 0);
    assert_eq!(7_u32.register_operation(ZeroIfZero, 2), 7);
    assert_eq!(7_u64.register_operation(ZeroIfNonzero, 0), 7);
    assert_eq!(7_u64.register_operation(ZeroIfNonzero, 2), 0);
}

fn unary<W: Arithmetic>(operation: ImmediateOperation, value: W) -> W {
    value.immediate_operation(operation, 0)
}

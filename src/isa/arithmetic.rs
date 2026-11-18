// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod aes;
mod bits;
mod sha;

use super::instruction::{
    AesByteOperation, ImmediateOperation, RegisterOperation,
};
use super::xlen::Word;

macro_rules! arithmetic {
    ($word:ty, $signed:ty, $wide:ty, $wide_signed:ty) => {
        impl Arithmetic for $word {
            fn immediate_operation(
                self,
                operation: ImmediateOperation,
                immediate: i32,
            ) -> Self {
                use ImmediateOperation::*;
                let first = self;
                let value = immediate as $signed;
                let shift = immediate as u32 & (<$word>::BITS - 1);
                let low = first.to_u64() as u32;
                match operation {
                    Add => first.wrapping_add(value as $word),
                    SetLessThan => <$word>::from((first as $signed) < value),
                    SetLessThanUnsigned => {
                        <$word>::from(first < value as $word)
                    }
                    Xor => first ^ value as $word,
                    Or => first | value as $word,
                    And => first & value as $word,
                    ShiftLeft => first.wrapping_shl(shift),
                    ShiftRightLogical => first.wrapping_shr(shift),
                    ShiftRightArithmetic => {
                        ((first as $signed) >> shift) as $word
                    }
                    RotateRight => first.rotate_right(shift),
                    BitClear => first & !(1 << shift),
                    BitExtract => (first >> shift) & 1,
                    BitInvert => first ^ (1 << shift),
                    BitSet => first | (1 << shift),
                    ShiftLeftUnsignedWord => {
                        <$word>::from_u64(first.to_u64() & 0xffff_ffff)
                            .wrapping_shl(shift)
                    }
                    CountLeadingZeros => {
                        <$word>::from_u64(first.leading_zeros().into())
                    }
                    CountTrailingZeros => {
                        <$word>::from_u64(first.trailing_zeros().into())
                    }
                    CountPopulation => {
                        <$word>::from_u64(first.count_ones().into())
                    }
                    SignExtendByte => first as i8 as $signed as $word,
                    SignExtendHalfword => first as i16 as $signed as $word,
                    OrCombineBytes => {
                        <$word>::from_u64(bits::or_combine(first.to_u64()))
                    }
                    ByteReverse => first.swap_bytes(),
                    BitReverseBytes => first.reverse_bits().swap_bytes(),
                    Zip => <$word>::from_u64(
                        bits::zip(first.to_u64() as u32).into(),
                    ),
                    Unzip => <$word>::from_u64(
                        bits::unzip(first.to_u64() as u32).into(),
                    ),
                    Sha256Sigma0 => {
                        <$word>::from_i32(sha::sha256_sigma0(low) as i32)
                    }
                    Sha256Sigma1 => {
                        <$word>::from_i32(sha::sha256_sigma1(low) as i32)
                    }
                    Sha256Sum0 => {
                        <$word>::from_i32(sha::sha256_sum0(low) as i32)
                    }
                    Sha256Sum1 => {
                        <$word>::from_i32(sha::sha256_sum1(low) as i32)
                    }
                    Sha512Sigma0 => {
                        <$word>::from_u64(sha::sha512_sigma0(first.to_u64()))
                    }
                    Sha512Sigma1 => {
                        <$word>::from_u64(sha::sha512_sigma1(first.to_u64()))
                    }
                    Sha512Sum0 => {
                        <$word>::from_u64(sha::sha512_sum0(first.to_u64()))
                    }
                    Sha512Sum1 => {
                        <$word>::from_u64(sha::sha512_sum1(first.to_u64()))
                    }
                    AesInverseMix => {
                        <$word>::from_u64(aes::inverse_mix(first.to_u64()))
                    }
                    AesKeySchedule1 => <$word>::from_u64(aes::key_schedule_1(
                        first.to_u64(),
                        immediate as u32,
                    )),
                }
            }

            fn register_operation(
                self,
                operation: RegisterOperation,
                second: Self,
            ) -> Self {
                use RegisterOperation::*;
                const BITS: u32 = <$word>::BITS;
                const LOWEST: $word = 1 << (BITS - 1);
                let first = self;
                let shift = second as u32 & (BITS - 1);
                match operation {
                    Add => first.wrapping_add(second),
                    Sub => first.wrapping_sub(second),
                    ShiftLeft => first.wrapping_shl(shift),
                    SetLessThan => {
                        <$word>::from((first as $signed) < (second as $signed))
                    }
                    SetLessThanUnsigned => <$word>::from(first < second),
                    Xor => first ^ second,
                    ShiftRightLogical => first.wrapping_shr(shift),
                    ShiftRightArithmetic => {
                        ((first as $signed) >> shift) as $word
                    }
                    Or => first | second,
                    And => first & second,
                    Mul => first.wrapping_mul(second),
                    MulHighSigned => {
                        (((first as $signed as $wide_signed)
                            * (second as $signed as $wide_signed))
                            >> BITS) as $word
                    }
                    MulHighSignedUnsigned => {
                        (((first as $signed as $wide_signed)
                            * (second as $wide as $wide_signed))
                            >> BITS) as $word
                    }
                    MulHighUnsigned => {
                        (((first as $wide) * (second as $wide)) >> BITS)
                            as $word
                    }
                    DivSigned | DivUnsigned if second == 0 => <$word>::MAX,
                    DivSigned if first == LOWEST && second == <$word>::MAX => {
                        first
                    }
                    DivSigned => {
                        ((first as $signed) / (second as $signed)) as $word
                    }
                    DivUnsigned => first / second,
                    RemainderSigned | RemainderUnsigned if second == 0 => first,
                    RemainderSigned
                        if first == LOWEST && second == <$word>::MAX =>
                    {
                        0
                    }
                    RemainderSigned => {
                        ((first as $signed) % (second as $signed)) as $word
                    }
                    RemainderUnsigned => first % second,
                    operation => first.bit_operation(operation, second),
                }
            }

            fn bit_operation(
                self,
                operation: RegisterOperation,
                second: Self,
            ) -> Self {
                use RegisterOperation::*;
                const BITS: u32 = <$word>::BITS;
                let first = self;
                let shift = second as u32 & (BITS - 1);
                let unsigned_word =
                    <$word>::from_u64(first.to_u64() & 0xffff_ffff);
                let (wide, other) = (first.to_u64(), second.to_u64());
                let (low, other_low) = (wide as u32, other as u32);
                let product = || bits::carryless(wide, other);
                let narrow = |value: u32| <$word>::from_u64(value.into());
                match operation {
                    AndNot => first & !second,
                    OrNot => first | !second,
                    XorNot => !(first ^ second),
                    Max => (first as $signed).max(second as $signed) as $word,
                    MaxUnsigned => first.max(second),
                    Min => (first as $signed).min(second as $signed) as $word,
                    MinUnsigned => first.min(second),
                    RotateLeft => first.rotate_left(shift),
                    RotateRight => first.rotate_right(shift),
                    ShiftLeft1Add => (first << 1).wrapping_add(second),
                    ShiftLeft2Add => (first << 2).wrapping_add(second),
                    ShiftLeft3Add => (first << 3).wrapping_add(second),
                    AddUnsignedWord => unsigned_word.wrapping_add(second),
                    ShiftLeft1AddUnsignedWord => {
                        (unsigned_word << 1).wrapping_add(second)
                    }
                    ShiftLeft2AddUnsignedWord => {
                        (unsigned_word << 2).wrapping_add(second)
                    }
                    ShiftLeft3AddUnsignedWord => {
                        (unsigned_word << 3).wrapping_add(second)
                    }
                    BitClear => first & !(1 << shift),
                    BitExtract => (first >> shift) & 1,
                    BitInvert => first ^ (1 << shift),
                    BitSet => first | (1 << shift),
                    CarrylessMul => product() as $word,
                    CarrylessMulHigh => (product() >> BITS) as $word,
                    CarrylessMulReversed => (product() >> (BITS - 1)) as $word,
                    Pack => {
                        (second << (BITS / 2))
                            | ((first << (BITS / 2)) >> (BITS / 2))
                    }
                    PackHigh => ((second & 0xff) << 8) | (first & 0xff),
                    Permute4 => {
                        <$word>::from_u64(bits::permute(wide, other, 4, BITS))
                    }
                    Permute8 => {
                        <$word>::from_u64(bits::permute(wide, other, 8, BITS))
                    }
                    ZeroIfZero if second == 0 => 0,
                    ZeroIfNonzero if second != 0 => 0,
                    ZeroIfZero | ZeroIfNonzero => first,
                    AesDecrypt => <$word>::from_u64(aes::decrypt(wide, other)),
                    AesDecryptMix => {
                        <$word>::from_u64(aes::decrypt_mix(wide, other))
                    }
                    AesEncrypt => <$word>::from_u64(aes::encrypt(wide, other)),
                    AesEncryptMix => {
                        <$word>::from_u64(aes::encrypt_mix(wide, other))
                    }
                    AesKeySchedule2 => {
                        <$word>::from_u64(aes::key_schedule_2(wide, other))
                    }
                    Sha512Sigma0High => {
                        narrow(sha::sha512_sigma0_high(low, other_low))
                    }
                    Sha512Sigma0Low => {
                        narrow(sha::sha512_sigma0_low(low, other_low))
                    }
                    Sha512Sigma1High => {
                        narrow(sha::sha512_sigma1_high(low, other_low))
                    }
                    Sha512Sigma1Low => {
                        narrow(sha::sha512_sigma1_low(low, other_low))
                    }
                    Sha512Sum0Rotated => {
                        narrow(sha::sha512_sum0_rotated(low, other_low))
                    }
                    Sha512Sum1Rotated => {
                        narrow(sha::sha512_sum1_rotated(low, other_low))
                    }
                    _ => first,
                }
            }
        }
    };
}

arithmetic!(u32, i32, u64, i64);
arithmetic!(u64, i64, u128, i128);

pub trait Arithmetic: Sized {
    fn bit_operation(self, operation: RegisterOperation, second: Self) -> Self;

    fn immediate_operation(
        self,
        operation: ImmediateOperation,
        immediate: i32,
    ) -> Self;

    fn register_operation(
        self,
        operation: RegisterOperation,
        second: Self,
    ) -> Self;
}

pub(crate) fn perform_aes_byte_operation(
    operation: AesByteOperation,
    first: u32,
    second: u32,
    select: u8,
) -> u32 {
    aes::byte(operation, first, second, select)
}

pub(crate) fn perform_immediate_operation<W: Word>(
    operation: ImmediateOperation,
    first: W,
    immediate: i32,
) -> W {
    first.immediate_operation(operation, immediate)
}

pub(crate) fn perform_register_operation<W: Word>(
    operation: RegisterOperation,
    first: W,
    second: W,
) -> W {
    first.register_operation(operation, second)
}

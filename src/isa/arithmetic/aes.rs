// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::instruction::AesByteOperation;

const FORWARD: [u8; 256] = forward_box();
const INVERSE: [u8; 256] = inverse_box();

const ROUND_CONSTANTS: [u32; 11] =
    [1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36, 0];

pub(super) fn byte(
    operation: AesByteOperation,
    first: u32,
    second: u32,
    select: u8,
) -> u32 {
    use AesByteOperation::*;
    let shift = u32::from(select) * 8;
    let input = (second >> shift) as u8;
    let column = match operation {
        Decrypt => u32::from(INVERSE[usize::from(input)]),
        DecryptMix => {
            inverse_mix_column(u32::from(INVERSE[usize::from(input)]))
        }
        Encrypt => u32::from(FORWARD[usize::from(input)]),
        EncryptMix => mix_column(u32::from(FORWARD[usize::from(input)])),
    };
    first ^ column.rotate_left(shift)
}

fn columns(value: u64, mix: fn(u32) -> u32) -> u64 {
    (u64::from(mix((value >> 32) as u32)) << 32) | u64::from(mix(value as u32))
}

pub(super) fn decrypt(first: u64, second: u64) -> u64 {
    substitute(shift_rows_inverse(first, second), &INVERSE)
}

pub(super) fn decrypt_mix(first: u64, second: u64) -> u64 {
    inverse_mix(decrypt(first, second))
}

pub(super) fn encrypt(first: u64, second: u64) -> u64 {
    substitute(shift_rows_forward(first, second), &FORWARD)
}

pub(super) fn encrypt_mix(first: u64, second: u64) -> u64 {
    columns(encrypt(first, second), mix_column)
}

const fn forward_box() -> [u8; 256] {
    let mut table = [0; 256];
    let mut value = 0;
    while value < 256 {
        let mut inverse = 1u8;
        let mut square = value as u8;
        let mut exponent = 254;
        while exponent != 0 {
            if exponent & 1 != 0 {
                inverse = multiply(inverse, square);
            }
            square = multiply(square, square);
            exponent >>= 1;
        }
        table[value] = inverse
            ^ inverse.rotate_left(1)
            ^ inverse.rotate_left(2)
            ^ inverse.rotate_left(3)
            ^ inverse.rotate_left(4)
            ^ 0x63;
        value += 1;
    }
    table
}

const fn inverse_box() -> [u8; 256] {
    let forward = forward_box();
    let mut table = [0; 256];
    let mut value = 0;
    while value < 256 {
        table[forward[value] as usize] = value as u8;
        value += 1;
    }
    table
}

pub(super) fn inverse_mix(value: u64) -> u64 {
    columns(value, inverse_mix_column)
}

fn inverse_mix_column(column: u32) -> u32 {
    mixed(column, [14, 11, 13, 9])
}

pub(super) fn key_schedule_1(value: u64, round: u32) -> u64 {
    let word = (value >> 32) as u32;
    let rotated = if round == 10 {
        word
    } else {
        word.rotate_right(8)
    };
    let substituted = u32::from_le_bytes(
        rotated.to_le_bytes().map(|byte| FORWARD[usize::from(byte)]),
    ) ^ ROUND_CONSTANTS[round as usize % 11];
    (u64::from(substituted) << 32) | u64::from(substituted)
}

pub(super) fn key_schedule_2(first: u64, second: u64) -> u64 {
    let low = (first >> 32) as u32 ^ second as u32;
    let high = low ^ (second >> 32) as u32;
    (u64::from(high) << 32) | u64::from(low)
}

fn mix_column(column: u32) -> u32 {
    mixed(column, [2, 3, 1, 1])
}

fn mixed(column: u32, factors: [u8; 4]) -> u32 {
    let bytes = column.to_le_bytes();
    u32::from_le_bytes(core::array::from_fn(|row| {
        (0..4).fold(0, |sum, index| {
            sum ^ multiply(bytes[(row + index) % 4], factors[index])
        })
    }))
}

const fn multiply(mut first: u8, mut second: u8) -> u8 {
    let mut product = 0;
    while second != 0 {
        if second & 1 != 0 {
            product ^= first;
        }
        first = (first << 1) ^ if first & 0x80 != 0 { 0x1b } else { 0 };
        second >>= 1;
    }
    product
}

fn picked(first: u64, second: u64, order: [usize; 8]) -> u64 {
    let state = ((u128::from(second) << 64) | u128::from(first)).to_le_bytes();
    u64::from_le_bytes(order.map(|index| state[index]))
}

fn shift_rows_forward(first: u64, second: u64) -> u64 {
    picked(first, second, [0, 5, 10, 15, 4, 9, 14, 3])
}

fn shift_rows_inverse(first: u64, second: u64) -> u64 {
    picked(first, second, [0, 13, 10, 7, 4, 1, 14, 11])
}

fn substitute(value: u64, table: &[u8; 256]) -> u64 {
    u64::from_le_bytes(value.to_le_bytes().map(|byte| table[usize::from(byte)]))
}

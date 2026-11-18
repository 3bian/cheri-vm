// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub(super) fn carryless(first: u64, second: u64) -> u128 {
    (0..64)
        .filter(|bit| (second >> bit) & 1 != 0)
        .fold(0, |product, bit| product ^ (u128::from(first) << bit))
}

pub(super) fn or_combine(value: u64) -> u64 {
    u64::from_le_bytes(
        value
            .to_le_bytes()
            .map(|byte| if byte == 0 { 0 } else { 0xff }),
    )
}

pub(super) fn permute(value: u64, indexes: u64, width: u32, bits: u32) -> u64 {
    let mask = (1u64 << width) - 1;
    (0..bits / width).fold(0, |result, position| {
        let index = (indexes >> (position * width)) & mask;
        if index < u64::from(bits / width) {
            result
                | (((value >> (index * u64::from(width))) & mask)
                    << (position * width))
        } else {
            result
        }
    })
}

pub(super) fn unzip(value: u32) -> u32 {
    (0..16).fold(0, |result, bit| {
        result
            | (((value >> (2 * bit)) & 1) << bit)
            | (((value >> (2 * bit + 1)) & 1) << (bit + 16))
    })
}

pub(super) fn zip(value: u32) -> u32 {
    (0..16).fold(0, |result, bit| {
        result
            | (((value >> bit) & 1) << (2 * bit))
            | (((value >> (bit + 16)) & 1) << (2 * bit + 1))
    })
}

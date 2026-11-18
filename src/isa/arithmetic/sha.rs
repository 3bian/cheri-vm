// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub(super) fn sha256_sigma0(value: u32) -> u32 {
    value.rotate_right(7) ^ value.rotate_right(18) ^ (value >> 3)
}

pub(super) fn sha256_sigma1(value: u32) -> u32 {
    value.rotate_right(17) ^ value.rotate_right(19) ^ (value >> 10)
}

pub(super) fn sha256_sum0(value: u32) -> u32 {
    value.rotate_right(2) ^ value.rotate_right(13) ^ value.rotate_right(22)
}

pub(super) fn sha256_sum1(value: u32) -> u32 {
    value.rotate_right(6) ^ value.rotate_right(11) ^ value.rotate_right(25)
}

pub(super) fn sha512_sigma0(value: u64) -> u64 {
    value.rotate_right(1) ^ value.rotate_right(8) ^ (value >> 7)
}

pub(super) fn sha512_sigma0_high(first: u32, second: u32) -> u32 {
    (first >> 1) ^ (first >> 7) ^ (first >> 8) ^ (second << 31) ^ (second << 24)
}

pub(super) fn sha512_sigma0_low(first: u32, second: u32) -> u32 {
    sha512_sigma0_high(first, second) ^ (second << 25)
}

pub(super) fn sha512_sigma1(value: u64) -> u64 {
    value.rotate_right(19) ^ value.rotate_right(61) ^ (value >> 6)
}

pub(super) fn sha512_sigma1_high(first: u32, second: u32) -> u32 {
    (first << 3)
        ^ (first >> 6)
        ^ (first >> 19)
        ^ (second >> 29)
        ^ (second << 13)
}

pub(super) fn sha512_sigma1_low(first: u32, second: u32) -> u32 {
    sha512_sigma1_high(first, second) ^ (second << 26)
}

pub(super) fn sha512_sum0(value: u64) -> u64 {
    value.rotate_right(28) ^ value.rotate_right(34) ^ value.rotate_right(39)
}

pub(super) fn sha512_sum0_rotated(first: u32, second: u32) -> u32 {
    (first << 25)
        ^ (first << 30)
        ^ (first >> 28)
        ^ (second >> 7)
        ^ (second >> 2)
        ^ (second << 4)
}

pub(super) fn sha512_sum1(value: u64) -> u64 {
    value.rotate_right(14) ^ value.rotate_right(18) ^ value.rotate_right(41)
}

pub(super) fn sha512_sum1_rotated(first: u32, second: u32) -> u32 {
    (first << 23)
        ^ (first >> 14)
        ^ (first >> 18)
        ^ (second >> 9)
        ^ (second << 18)
        ^ (second << 14)
}

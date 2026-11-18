// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::flags::{INEXACT, INVALID};
use super::round::{RoundingMode, pack};
use super::value::{Class, Format, unpack};

fn extended(value: u64, width: u32) -> u64 {
    if width == 32 {
        i64::from(value as i32) as u64
    } else {
        value
    }
}

pub fn from_integer(
    format: Format,
    value: u64,
    signed: bool,
    width: u32,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    let value = match (signed, width) {
        (true, 32) => i64::from(value as i32) as u64,
        (false, 32) => u64::from(value as u32),
        _ => value,
    };
    let sign = signed && (value as i64) < 0;
    let magnitude = if sign { value.wrapping_neg() } else { value };
    if magnitude == 0 {
        return 0;
    }
    let leading = magnitude.leading_zeros();
    pack(
        format,
        sign,
        63 - leading as i32,
        magnitude << leading,
        false,
        mode,
        flags,
    )
}

pub fn resized(
    from: Format,
    to: Format,
    bits: u64,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    let value = unpack(from, bits);
    match value.class {
        Class::Signaling => {
            *flags |= INVALID;
            to.canonical_nan()
        }
        Class::Quiet => to.canonical_nan(),
        Class::Infinite => to.infinity(value.sign),
        Class::Zero => to.zero(value.sign),
        Class::Finite => pack(
            to,
            value.sign,
            value.exponent,
            value.significand,
            false,
            mode,
            flags,
        ),
    }
}

pub fn to_integer(
    format: Format,
    bits: u64,
    signed: bool,
    width: u32,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    let value = unpack(format, bits);
    let highest = if signed {
        (1_u128 << (width - 1)) - 1
    } else {
        (1_u128 << width) - 1
    };
    let lowest = if signed { 1_u128 << (width - 1) } else { 0 };
    let saturated = |negative: bool, flags: &mut u8| {
        *flags |= INVALID;
        extended(if negative { lowest } else { highest } as u64, width)
    };
    let (kept, half, rest) = match value.class {
        Class::Quiet | Class::Signaling => return saturated(false, flags),
        Class::Infinite => return saturated(value.sign, flags),
        Class::Zero => return 0,
        Class::Finite if value.exponent > 63 => {
            return saturated(value.sign, flags);
        }
        Class::Finite if value.exponent < -1 => (0, false, true),
        Class::Finite if value.exponent == -1 => {
            (0, true, value.significand << 1 != 0)
        }
        Class::Finite if value.exponent == 63 => {
            (value.significand, false, false)
        }
        Class::Finite => {
            let shift = 63 - value.exponent as u32;
            (
                value.significand >> shift,
                (value.significand >> (shift - 1)) & 1 == 1,
                value.significand & ((1 << (shift - 1)) - 1) != 0,
            )
        }
    };
    let up = mode.rounds_up(value.sign, kept & 1 == 1, half, rest);
    let magnitude = u128::from(kept) + u128::from(up);
    let limit = if value.sign { lowest } else { highest };
    if magnitude > limit {
        return saturated(value.sign, flags);
    }
    if half || rest {
        *flags |= INEXACT;
    }
    let magnitude = magnitude as u64;
    extended(
        if value.sign {
            magnitude.wrapping_neg()
        } else {
            magnitude
        },
        width,
    )
}

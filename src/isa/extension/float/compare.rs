// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::flags::INVALID;
use super::value::{Class, Format, unpack};

fn chosen(
    format: Format,
    first: u64,
    second: u64,
    largest: bool,
    flags: &mut u8,
) -> u64 {
    let nan = |bits| unpack(format, bits).is_nan();
    if unordered(format, first, second, true, flags) {
        return match (nan(first), nan(second)) {
            (true, true) => format.canonical_nan(),
            (true, false) => second,
            _ => first,
        };
    }
    let (low, high) = (ordered(format, first), ordered(format, second));
    let negative = first & format.sign() != 0;
    let takes_first = if low == high {
        negative != largest
    } else {
        (low < high) != largest
    };
    if takes_first { first } else { second }
}

pub fn classify(format: Format, bits: u64) -> u64 {
    let value = unpack(format, bits);
    let subnormal =
        (bits >> format.fraction_bits) & format.highest_exponent() == 0;
    1 << match (value.class, value.sign) {
        (Class::Infinite, true) => 0,
        (Class::Finite, true) if !subnormal => 1,
        (Class::Finite, true) => 2,
        (Class::Zero, true) => 3,
        (Class::Zero, false) => 4,
        (Class::Finite, false) if subnormal => 5,
        (Class::Finite, false) => 6,
        (Class::Infinite, false) => 7,
        (Class::Signaling, _) => 8,
        (Class::Quiet, _) => 9,
    }
}

pub fn equal(format: Format, first: u64, second: u64, flags: &mut u8) -> bool {
    !unordered(format, first, second, true, flags)
        && ordered(format, first) == ordered(format, second)
}

pub fn less_or_equal(
    format: Format,
    first: u64,
    second: u64,
    flags: &mut u8,
) -> bool {
    !unordered(format, first, second, false, flags)
        && ordered(format, first) <= ordered(format, second)
}

pub fn less_than(
    format: Format,
    first: u64,
    second: u64,
    flags: &mut u8,
) -> bool {
    !unordered(format, first, second, false, flags)
        && ordered(format, first) < ordered(format, second)
}

pub fn maximum(format: Format, first: u64, second: u64, flags: &mut u8) -> u64 {
    chosen(format, first, second, true, flags)
}

pub fn minimum(format: Format, first: u64, second: u64, flags: &mut u8) -> u64 {
    chosen(format, first, second, false, flags)
}

fn ordered(format: Format, bits: u64) -> i64 {
    let magnitude = (bits & (format.sign() - 1)) as i64;
    if bits & format.sign() != 0 {
        -magnitude
    } else {
        magnitude
    }
}

fn unordered(
    format: Format,
    first: u64,
    second: u64,
    quiet: bool,
    flags: &mut u8,
) -> bool {
    let classes = [unpack(format, first).class, unpack(format, second).class];
    let signaling = classes.contains(&Class::Signaling);
    let any = signaling || classes.contains(&Class::Quiet);
    if signaling || (any && !quiet) {
        *flags |= INVALID;
    }
    any
}

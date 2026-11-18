// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::flags::{DIVIDE_BY_ZERO, INVALID};
use super::host;
use super::round::{RoundingMode, pack};
use super::value::{Class, Format, Value, unpack};

struct Term {
    exponent: i32,
    sign: bool,
    significand: u128,
}

impl Term {
    fn of(value: &Value) -> Self {
        Self {
            exponent: value.exponent,
            sign: value.sign,
            significand: u128::from(value.significand) << 63,
        }
    }
}

pub fn add(
    format: Format,
    first: u64,
    second: u64,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    match host::sum(format, first, second, mode, flags) {
        Some(bits) => bits,
        None => soft_add(format, first, second, mode, flags),
    }
}

fn combined(
    format: Format,
    first: Term,
    second: Term,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    let (large, small) = if (first.exponent, first.significand)
        >= (second.exponent, second.significand)
    {
        (first, second)
    } else {
        (second, first)
    };
    let aligned = shifted_right(
        small.significand,
        (large.exponent - small.exponent) as u32,
    );
    let magnitude = if large.sign == small.sign {
        large.significand + aligned
    } else {
        large.significand - aligned
    };
    if magnitude == 0 {
        return format.zero(mode == RoundingMode::Down);
    }
    let leading = magnitude.leading_zeros();
    let normalized = magnitude << leading;
    pack(
        format,
        large.sign,
        large.exponent + 1 - leading as i32,
        (normalized >> 64) as u64,
        normalized as u64 != 0,
        mode,
        flags,
    )
}

pub fn divide(
    format: Format,
    first: u64,
    second: u64,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    match host::quotient(format, first, second, mode, flags) {
        Some(bits) => bits,
        None => soft_divide(format, first, second, mode, flags),
    }
}

fn exact(
    format: Format,
    value: &Value,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    pack(
        format,
        value.sign,
        value.exponent,
        value.significand,
        false,
        mode,
        flags,
    )
}

pub fn fused_multiply_add(
    format: Format,
    [first, second, addend]: [u64; 3],
    [negate_product, negate_addend]: [bool; 2],
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    use Class::*;
    let first = unpack(format, first);
    let second = unpack(format, second);
    let mut addend = unpack(format, addend);
    if negate_addend {
        addend = addend.negated();
    }
    let sign = (first.sign != second.sign) != negate_product;
    match (first.class, second.class, addend.class) {
        (Infinite, Zero, _) | (Zero, Infinite, _) => invalid(format, flags),
        (Quiet | Signaling, ..)
        | (_, Quiet | Signaling, _)
        | (.., Quiet | Signaling) => {
            not_a_number(format, &[first, second, addend], flags)
        }
        (Infinite, ..) | (_, Infinite, _)
            if addend.class == Infinite && addend.sign != sign =>
        {
            invalid(format, flags)
        }
        (Infinite, ..) | (_, Infinite, _) => format.infinity(sign),
        (.., Infinite) => format.infinity(addend.sign),
        (Zero, _, Zero) | (_, Zero, Zero) if addend.sign == sign => {
            format.zero(sign)
        }
        (Zero, _, Zero) | (_, Zero, Zero) => {
            format.zero(mode == RoundingMode::Down)
        }
        (Zero, ..) | (_, Zero, _) => exact(format, &addend, mode, flags),
        (Finite, Finite, _) => {
            let product =
                u128::from(first.significand) * u128::from(second.significand);
            let carried = product >> 127 != 0;
            let product = Term {
                exponent: first.exponent + second.exponent + i32::from(carried),
                sign,
                significand: shifted_right(product, u32::from(carried)),
            };
            if addend.class == Zero {
                let normalized = product.significand << 1;
                return pack(
                    format,
                    sign,
                    product.exponent,
                    (normalized >> 64) as u64,
                    normalized as u64 != 0,
                    mode,
                    flags,
                );
            }
            combined(format, product, Term::of(&addend), mode, flags)
        }
    }
}

fn invalid(format: Format, flags: &mut u8) -> u64 {
    *flags |= INVALID;
    format.canonical_nan()
}

pub fn multiply(
    format: Format,
    first: u64,
    second: u64,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    match host::product(format, first, second, mode, flags) {
        Some(bits) => bits,
        None => soft_multiply(format, first, second, mode, flags),
    }
}

fn not_a_number(format: Format, values: &[Value], flags: &mut u8) -> u64 {
    if values.iter().any(|value| value.class == Class::Signaling) {
        *flags |= INVALID;
    }
    format.canonical_nan()
}

fn shifted_right(value: u128, count: u32) -> u128 {
    match count {
        0 => value,
        1..=127 => (value >> count) | u128::from(value << (128 - count) != 0),
        _ => u128::from(value != 0),
    }
}

pub fn soft_add(
    format: Format,
    first: u64,
    second: u64,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    sum(
        format,
        unpack(format, first),
        unpack(format, second),
        mode,
        flags,
    )
}

pub fn soft_divide(
    format: Format,
    first: u64,
    second: u64,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    use Class::*;
    let (first, second) = (unpack(format, first), unpack(format, second));
    let sign = first.sign != second.sign;
    match (first.class, second.class) {
        (Quiet | Signaling, _) | (_, Quiet | Signaling) => {
            not_a_number(format, &[first, second], flags)
        }
        (Infinite, Infinite) | (Zero, Zero) => invalid(format, flags),
        (Infinite, _) => format.infinity(sign),
        (_, Infinite) | (Zero, _) => format.zero(sign),
        (Finite, Zero) => {
            *flags |= DIVIDE_BY_ZERO;
            format.infinity(sign)
        }
        (Finite, Finite) => {
            let numerator = u128::from(first.significand) << 64;
            let divisor = u128::from(second.significand);
            let quotient = numerator / divisor;
            let remainder = numerator % divisor != 0;
            let wide = quotient >> 64 != 0;
            let (significand, sticky) = if wide {
                ((quotient >> 1) as u64, quotient & 1 != 0 || remainder)
            } else {
                (quotient as u64, remainder)
            };
            pack(
                format,
                sign,
                first.exponent - second.exponent - i32::from(!wide),
                significand,
                sticky,
                mode,
                flags,
            )
        }
    }
}

pub fn soft_multiply(
    format: Format,
    first: u64,
    second: u64,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    use Class::*;
    let (first, second) = (unpack(format, first), unpack(format, second));
    let sign = first.sign != second.sign;
    match (first.class, second.class) {
        (Quiet | Signaling, _) | (_, Quiet | Signaling) => {
            not_a_number(format, &[first, second], flags)
        }
        (Infinite, Zero) | (Zero, Infinite) => invalid(format, flags),
        (Infinite, _) | (_, Infinite) => format.infinity(sign),
        (Zero, _) | (_, Zero) => format.zero(sign),
        (Finite, Finite) => {
            let product =
                u128::from(first.significand) * u128::from(second.significand);
            let leading = product.leading_zeros();
            let normalized = product << leading;
            pack(
                format,
                sign,
                first.exponent + second.exponent + 1 - leading as i32,
                (normalized >> 64) as u64,
                normalized as u64 != 0,
                mode,
                flags,
            )
        }
    }
}

pub fn soft_subtract(
    format: Format,
    first: u64,
    second: u64,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    sum(
        format,
        unpack(format, first),
        unpack(format, second).negated(),
        mode,
        flags,
    )
}

pub fn square_root(
    format: Format,
    value: u64,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    let value = unpack(format, value);
    match value.class {
        Class::Quiet | Class::Signaling => {
            not_a_number(format, &[value], flags)
        }
        Class::Zero => format.zero(value.sign),
        _ if value.sign => invalid(format, flags),
        Class::Infinite => format.infinity(false),
        Class::Finite => {
            let odd = value.exponent & 1 != 0;
            let mut remainder =
                u128::from(value.significand) << if odd { 64 } else { 63 };
            let mut root = 0_u128;
            let mut bit = 1_u128 << 126;
            while bit != 0 {
                if remainder >= root + bit {
                    remainder -= root + bit;
                    root = (root >> 1) + bit;
                } else {
                    root >>= 1;
                }
                bit >>= 2;
            }
            pack(
                format,
                false,
                value.exponent.div_euclid(2),
                root as u64,
                remainder != 0,
                mode,
                flags,
            )
        }
    }
}

pub fn subtract(
    format: Format,
    first: u64,
    second: u64,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    match host::sum(format, first, second ^ format.sign(), mode, flags) {
        Some(bits) => bits,
        None => soft_subtract(format, first, second, mode, flags),
    }
}

fn sum(
    format: Format,
    first: Value,
    second: Value,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    use Class::*;
    match (first.class, second.class) {
        (Quiet | Signaling, _) | (_, Quiet | Signaling) => {
            not_a_number(format, &[first, second], flags)
        }
        (Infinite, Infinite) if first.sign != second.sign => {
            invalid(format, flags)
        }
        (Infinite, _) => format.infinity(first.sign),
        (_, Infinite) => format.infinity(second.sign),
        (Zero, Zero) if first.sign == second.sign => format.zero(first.sign),
        (Zero, Zero) => format.zero(mode == RoundingMode::Down),
        (Zero, Finite) => exact(format, &second, mode, flags),
        (Finite, Zero) => exact(format, &first, mode, flags),
        (Finite, Finite) => {
            combined(format, Term::of(&first), Term::of(&second), mode, flags)
        }
    }
}

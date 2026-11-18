// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use core::ops::{Add, Div, Mul, Sub};

use super::flags::INEXACT;
use super::round::RoundingMode;
use super::value::Format;

impl Host for f32 {
    const FORMAT: Format = Format::SINGLE;
    const ZERO: Self = 0.0;

    fn bits(self) -> u64 {
        self.to_bits().into()
    }

    fn of(bits: u64) -> Self {
        Self::from_bits(bits as u32)
    }
}

impl Host for f64 {
    const FORMAT: Format = Format::DOUBLE;
    const ZERO: Self = 0.0;

    fn bits(self) -> u64 {
        self.to_bits()
    }

    fn of(bits: u64) -> Self {
        Self::from_bits(bits)
    }
}

trait Host:
    Add<Output = Self>
    + Copy
    + Div<Output = Self>
    + Mul<Output = Self>
    + PartialEq
    + Sub<Output = Self>
{
    const FORMAT: Format;
    const ZERO: Self;

    fn bits(self) -> u64;

    fn of(bits: u64) -> Self;
}

#[inline]
fn eligible(
    format: Format,
    first: u64,
    second: u64,
    mode: RoundingMode,
) -> bool {
    mode == RoundingMode::NearestEven
        && normal(format, first)
        && normal(format, second)
}

#[inline]
fn field(format: Format, bits: u64) -> u64 {
    (bits >> format.fraction_bits) & format.highest_exponent()
}

#[inline]
fn normal(format: Format, bits: u64) -> bool {
    let field = field(format, bits);
    field != 0 && field != format.highest_exponent()
}

#[inline]
pub(crate) fn product(
    format: Format,
    first: u64,
    second: u64,
    mode: RoundingMode,
    flags: &mut u8,
) -> Option<u64> {
    if !eligible(format, first, second, mode) {
        None
    } else if format == Format::DOUBLE {
        product_as::<f64>(first, second, flags)
    } else if format == Format::SINGLE {
        product_as::<f32>(first, second, flags)
    } else {
        None
    }
}

#[inline]
fn product_as<T: Host>(first: u64, second: u64, flags: &mut u8) -> Option<u64> {
    let format = T::FORMAT;
    let bits = (T::of(first) * T::of(second)).bits();
    if !settled(format, bits) {
        return None;
    }
    let whole = significand(format, first) * significand(format, second);
    let width = format.fraction_bits + 1;
    let dropped = if whole >> (2 * width - 1) != 0 {
        width
    } else {
        width - 1
    };
    if whole & ((1 << dropped) - 1) != 0 {
        *flags |= INEXACT;
    }
    Some(bits)
}

#[inline]
pub(crate) fn quotient(
    format: Format,
    first: u64,
    second: u64,
    mode: RoundingMode,
    flags: &mut u8,
) -> Option<u64> {
    if !eligible(format, first, second, mode) {
        None
    } else if format == Format::DOUBLE {
        quotient_as::<f64>(first, second, flags)
    } else if format == Format::SINGLE {
        quotient_as::<f32>(first, second, flags)
    } else {
        None
    }
}

#[inline]
fn quotient_as<T: Host>(
    first: u64,
    second: u64,
    flags: &mut u8,
) -> Option<u64> {
    let format = T::FORMAT;
    let bits = (T::of(first) / T::of(second)).bits();
    if !settled(format, bits) {
        return None;
    }
    let (numerator, divisor) =
        (significand(format, first), significand(format, second));
    let shift = if numerator >= divisor {
        format.fraction_bits
    } else {
        format.fraction_bits + 1
    };
    if significand(format, bits) * divisor != numerator << shift {
        *flags |= INEXACT;
    }
    Some(bits)
}

#[inline]
fn settled(format: Format, bits: u64) -> bool {
    let field = field(format, bits);
    field > 1 && field != format.highest_exponent()
}

#[inline]
fn significand(format: Format, bits: u64) -> u128 {
    u128::from((bits & format.fraction()) | (1 << format.fraction_bits))
}

#[inline]
pub(crate) fn sum(
    format: Format,
    first: u64,
    second: u64,
    mode: RoundingMode,
    flags: &mut u8,
) -> Option<u64> {
    if !eligible(format, first, second, mode) {
        None
    } else if format == Format::DOUBLE {
        sum_as::<f64>(first, second, flags)
    } else if format == Format::SINGLE {
        sum_as::<f32>(first, second, flags)
    } else {
        None
    }
}

#[inline]
fn sum_as<T: Host>(first: u64, second: u64, flags: &mut u8) -> Option<u64> {
    let (a, b) = (T::of(first), T::of(second));
    let sum = a + b;
    let bits = sum.bits();
    if !settled(T::FORMAT, bits) {
        return None;
    }
    let other = sum - a;
    let error = (a - (sum - other)) + (b - other);
    if error != T::ZERO {
        *flags |= INEXACT;
    }
    Some(bits)
}

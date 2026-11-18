// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::flags::{INEXACT, OVERFLOW, UNDERFLOW};
use super::value::Format;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoundingMode {
    Down,
    NearestAway,
    NearestEven,
    TowardZero,
    Up,
}

impl RoundingMode {
    pub fn from_bits(bits: u32) -> Option<Self> {
        Some(match bits {
            0 => Self::NearestEven,
            1 => Self::TowardZero,
            2 => Self::Down,
            3 => Self::Up,
            4 => Self::NearestAway,
            _ => return None,
        })
    }

    pub(super) fn rounds_up(
        self,
        sign: bool,
        odd: bool,
        half: bool,
        rest: bool,
    ) -> bool {
        match self {
            Self::Down => sign && (half || rest),
            Self::NearestAway => half,
            Self::NearestEven => half && (rest || odd),
            Self::TowardZero => false,
            Self::Up => !sign && (half || rest),
        }
    }
}

pub(super) fn pack(
    format: Format,
    sign: bool,
    exponent: i32,
    significand: u64,
    sticky: bool,
    mode: RoundingMode,
    flags: &mut u8,
) -> u64 {
    let precision = format.fraction_bits + 1;
    let dropped = 64 - precision;
    let lowest = 1 - format.bias();
    let (kept, half, rest) = split(significand, sticky, dropped);
    let carries = mode.rounds_up(sign, kept & 1 == 1, half, rest)
        && kept + 1 == 1 << precision;

    if exponent < lowest {
        let tiny = !(exponent == lowest - 1 && carries);
        let shift = dropped.saturating_add((lowest - exponent) as u32);
        let (mut kept, half, rest) = split(significand, sticky, shift);
        if mode.rounds_up(sign, kept & 1 == 1, half, rest) {
            kept += 1;
        }
        if half || rest {
            *flags |= INEXACT | if tiny { UNDERFLOW } else { 0 };
        }
        return format.zero(sign) | kept;
    }

    let (kept, exponent) = if carries {
        (1 << format.fraction_bits, exponent + 1)
    } else if mode.rounds_up(sign, kept & 1 == 1, half, rest) {
        (kept + 1, exponent)
    } else {
        (kept, exponent)
    };
    if exponent > format.bias() {
        *flags |= OVERFLOW | INEXACT;
        let infinite = match mode {
            RoundingMode::NearestAway | RoundingMode::NearestEven => true,
            RoundingMode::TowardZero => false,
            RoundingMode::Down => sign,
            RoundingMode::Up => !sign,
        };
        return if infinite {
            format.infinity(sign)
        } else {
            format.infinity(sign) - 1
        };
    }
    if half || rest {
        *flags |= INEXACT;
    }
    format.zero(sign)
        | (((exponent + format.bias()) as u64) << format.fraction_bits)
        | (kept & format.fraction())
}

fn split(significand: u64, sticky: bool, dropped: u32) -> (u64, bool, bool) {
    match dropped {
        0 => (significand, false, sticky),
        1..=63 => (
            significand >> dropped,
            (significand >> (dropped - 1)) & 1 == 1,
            significand & ((1 << (dropped - 1)) - 1) != 0 || sticky,
        ),
        64 => (0, significand >> 63 == 1, significand << 1 != 0 || sticky),
        _ => (0, false, significand != 0 || sticky),
    }
}

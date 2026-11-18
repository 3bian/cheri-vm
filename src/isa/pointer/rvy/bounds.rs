// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

const BASE_EXPONENT: u64 = 7;
const BASE_FIELD: u64 = 0x7ff << 3;
const EXPONENT_FORMAT: u64 = 1 << 26;
pub(super) const EXPONENT_LIMIT: u32 = 52;

pub(super) const FIELDS: u64 =
    BASE_FIELD | BASE_EXPONENT | EXPONENT_FORMAT | TOP_EXPONENT | TOP_FIELD;

const MANTISSA: u32 = 14;
const MANTISSA_MASK: u64 = (1 << MANTISSA) - 1;
const TOP_EXPONENT: u64 = 7 << 14;
const TOP_FIELD: u64 = 0x1ff << 17;

pub(super) struct Decoded {
    pub base: u64,
    pub exponent: u32,
    pub internal: bool,
    pub top: u128,
}

pub(super) fn decoded(address: u64, metadata: u64) -> Option<Decoded> {
    let internal = metadata & EXPONENT_FORMAT == 0;
    let low = (metadata >> 14) & 7;
    let base_low = metadata & 7;
    let mut top = ((metadata >> 17) & 0x1ff) << 3;
    let mut base = ((metadata >> 3) & 0x7ff) << 3;
    let exponent = if internal {
        let stored = ((low << 3) | base_low) as u32;
        if stored > EXPONENT_LIMIT {
            return None;
        }
        EXPONENT_LIMIT - stored
    } else {
        top |= low;
        base |= base_low;
        0
    };
    if internal
        && (exponent == EXPONENT_LIMIT && base != 0
            || exponent == EXPONENT_LIMIT - 1 && base >> 13 != 0)
    {
        return None;
    }
    let carry = u64::from(top < base & 0xfff);
    top |= (((base >> 12) + carry + u64::from(internal)) & 3) << 12;

    let middle = if exponent >= 64 {
        0
    } else {
        (address >> exponent) & MANTISSA_MASK
    };
    let limit = base.wrapping_sub(1 << (MANTISSA - 2)) & MANTISSA_MASK;
    let below = |value: u64| i128::from(value < limit);
    let upper = if exponent + MANTISSA >= 64 {
        0
    } else {
        i128::from(address >> (exponent + MANTISSA))
    };
    let placed = |mantissa: u64| {
        ((upper + below(mantissa) - below(middle)) << (exponent + MANTISSA))
            .wrapping_add(i128::from(mantissa) << exponent) as u128
    };
    let decoded_base = placed(base) as u64;
    let mut decoded_top = placed(top) & ((1 << 65) - 1);
    if exponent < EXPONENT_LIMIT - 1 {
        let wraps = decoded_top >> 63 & 1 == 0 && decoded_base >> 63 == 1;
        decoded_top =
            (decoded_top & u128::from(u64::MAX)) | (u128::from(wraps) << 64);
    }
    Some(Decoded {
        base: decoded_base,
        exponent,
        internal,
        top: decoded_top,
    })
}

pub(super) fn encoded(base: u64, length: u64) -> (u64, bool) {
    let top = u128::from(base) + u128::from(length);
    let wide_base = u128::from(base);
    let significant = (u128::from(length) >> (MANTISSA - 1)) as u64;
    let exponent = if significant == 0 {
        0
    } else {
        64 - significant.leading_zeros()
    };
    if exponent == 0 && length >> (MANTISSA - 2) & 1 == 0 {
        let fields = EXPONENT_FORMAT
            | ((top as u64 >> 3) & 0x1ff) << 17
            | (top as u64 & 7) << 14
            | (base & MANTISSA_MASK);
        return (fields, true);
    }
    let lost = |value: u128, shift: u32| value & ((1 << shift) - 1) != 0;
    let mut shift = exponent + 3;
    let mut exact = !lost(wide_base, shift) && !lost(top, shift);
    let rounded = |shift: u32| (top >> shift) + u128::from(lost(top, shift));
    if (rounded(shift) - (wide_base >> shift)) >> 10 & 1 == 1 {
        shift += 1;
        exact = !lost(wide_base, shift) && !lost(top, shift);
    }
    if shift - 3 >= EXPONENT_LIMIT {
        return (0, false);
    }
    let stored = u64::from(EXPONENT_LIMIT - (shift - 3));
    let fields = ((rounded(shift) as u64) & 0x1ff) << 17
        | (stored >> 3) << 14
        | ((wide_base >> shift) as u64 & 0x7ff) << 3
        | (stored & 7);
    (fields, exact)
}

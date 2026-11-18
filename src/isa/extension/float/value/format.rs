// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Format {
    pub exponent_bits: u32,
    pub fraction_bits: u32,
}

impl Format {
    pub const DOUBLE: Self = Self {
        exponent_bits: 11,
        fraction_bits: 52,
    };

    pub const SINGLE: Self = Self {
        exponent_bits: 8,
        fraction_bits: 23,
    };

    pub const fn bias(self) -> i32 {
        (1 << (self.exponent_bits - 1)) - 1
    }

    pub const fn canonical_nan(self) -> u64 {
        self.infinity(false) | self.quiet()
    }

    pub const fn fraction(self) -> u64 {
        (1 << self.fraction_bits) - 1
    }

    pub const fn highest_exponent(self) -> u64 {
        (1 << self.exponent_bits) - 1
    }

    pub const fn infinity(self, sign: bool) -> u64 {
        self.zero(sign) | (self.highest_exponent() << self.fraction_bits)
    }

    pub const fn quiet(self) -> u64 {
        1 << (self.fraction_bits - 1)
    }

    pub const fn sign(self) -> u64 {
        1 << (self.exponent_bits + self.fraction_bits)
    }

    pub const fn zero(self, sign: bool) -> u64 {
        if sign { self.sign() } else { 0 }
    }
}

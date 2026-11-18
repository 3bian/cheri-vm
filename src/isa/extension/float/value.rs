// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod class;
mod format;

pub(super) use class::Class;
pub use format::Format;

#[derive(Clone, Copy, Debug)]
pub(super) struct Value {
    pub(super) class: Class,
    pub(super) exponent: i32,
    pub(super) sign: bool,
    pub(super) significand: u64,
}

impl Value {
    pub(super) fn is_nan(&self) -> bool {
        matches!(self.class, Class::Quiet | Class::Signaling)
    }

    pub(super) fn negated(mut self) -> Self {
        self.sign = !self.sign;
        self
    }
}

pub(super) fn unpack(format: Format, bits: u64) -> Value {
    let sign = bits & format.sign() != 0;
    let field = (bits >> format.fraction_bits) & format.highest_exponent();
    let fraction = bits & format.fraction();
    let special = |class| Value {
        class,
        exponent: 0,
        sign,
        significand: 0,
    };
    if field == format.highest_exponent() {
        return special(if fraction == 0 {
            Class::Infinite
        } else if fraction & format.quiet() != 0 {
            Class::Quiet
        } else {
            Class::Signaling
        });
    }
    if field == 0 && fraction == 0 {
        return special(Class::Zero);
    }
    let (exponent, significand) = if field == 0 {
        let shift = fraction.leading_zeros();
        (
            64 - format.bias() - format.fraction_bits as i32 - shift as i32,
            fraction << shift,
        )
    } else {
        (
            field as i32 - format.bias(),
            (fraction | (1 << format.fraction_bits))
                << (63 - format.fraction_bits),
        )
    };
    Value {
        class: Class::Finite,
        exponent,
        sign,
        significand,
    }
}

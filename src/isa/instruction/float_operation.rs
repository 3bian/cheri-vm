// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod float_width;

pub use float_width::FloatWidth;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatOperation {
    Add,
    Classify,
    Divide,
    Equal,
    FromLong,
    FromUnsignedLong,
    FromUnsignedWord,
    FromWord,
    LessOrEqual,
    LessThan,
    Maximum,
    Minimum,
    MoveFromInteger,
    MoveToInteger,
    Multiply,
    MultiplyAdd,
    MultiplySubtract,
    NegatedMultiplyAdd,
    NegatedMultiplySubtract,
    Resize,
    SignInject,
    SignInjectNegated,
    SignInjectXor,
    SquareRoot,
    Subtract,
    ToLong,
    ToUnsignedLong,
    ToUnsignedWord,
    ToWord,
}

impl FloatOperation {
    pub(crate) const fn rounds(self) -> bool {
        !matches!(
            self,
            Self::Classify
                | Self::Equal
                | Self::LessOrEqual
                | Self::LessThan
                | Self::Maximum
                | Self::Minimum
                | Self::MoveFromInteger
                | Self::MoveToInteger
                | Self::SignInject
                | Self::SignInjectNegated
                | Self::SignInjectXor
        )
    }
}

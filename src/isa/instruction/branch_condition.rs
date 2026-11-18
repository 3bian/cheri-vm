// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::xlen::Word;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BranchCondition {
    Equal,
    GreaterOrEqual,
    GreaterOrEqualUnsigned,
    LessThan,
    LessThanUnsigned,
    NotEqual,
}

impl BranchCondition {
    pub(crate) fn test<W: Word>(self, lhs: W, rhs: W) -> bool {
        match self {
            Self::Equal => lhs == rhs,
            Self::NotEqual => lhs != rhs,
            Self::LessThan => lhs.to_i64() < rhs.to_i64(),
            Self::LessThanUnsigned => lhs < rhs,
            Self::GreaterOrEqual => lhs.to_i64() >= rhs.to_i64(),
            Self::GreaterOrEqualUnsigned => lhs >= rhs,
        }
    }
}

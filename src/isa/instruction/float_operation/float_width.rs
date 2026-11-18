// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::extension::float::Format;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatWidth {
    Double,
    Single,
}

impl FloatWidth {
    pub(crate) const fn bytes(self) -> usize {
        match self {
            Self::Double => 8,
            Self::Single => 4,
        }
    }

    pub(crate) const fn format(self) -> Format {
        match self {
            Self::Double => Format::DOUBLE,
            Self::Single => Format::SINGLE,
        }
    }
}

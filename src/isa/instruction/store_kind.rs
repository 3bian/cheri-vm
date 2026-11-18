// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreKind {
    Byte,
    Capability,
    Doubleword,
    Halfword,
    Word,
}

impl StoreKind {
    pub(crate) const fn width(self) -> usize {
        match self {
            Self::Byte => 1,
            Self::Halfword => 2,
            Self::Word => 4,
            Self::Doubleword => 8,
            Self::Capability => 16,
        }
    }
}

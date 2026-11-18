// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtomicWidth {
    Capability,
    Doubleword,
    Word,
}

impl AtomicWidth {
    pub(crate) const fn bytes(self) -> usize {
        match self {
            Self::Capability => 16,
            Self::Doubleword => 8,
            Self::Word => 4,
        }
    }
}

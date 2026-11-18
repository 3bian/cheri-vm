// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::xlen::Word;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadKind {
    Capability,
    Doubleword,
    SignedByte,
    SignedHalfword,
    UnsignedByte,
    UnsignedHalfword,
    UnsignedWord,
    Word,
}

impl LoadKind {
    pub(crate) fn extend<W: Word>(self, bits: u64) -> W {
        W::from_u64(match self {
            Self::Capability | Self::Doubleword => bits,
            Self::SignedByte => bits as i8 as u64,
            Self::SignedHalfword => bits as i16 as u64,
            Self::UnsignedByte => bits & 0xff,
            Self::UnsignedHalfword => bits & 0xffff,
            Self::UnsignedWord => bits & 0xffff_ffff,
            Self::Word => bits as i32 as u64,
        })
    }

    pub(crate) const fn width(self) -> usize {
        match self {
            Self::SignedByte | Self::UnsignedByte => 1,
            Self::SignedHalfword | Self::UnsignedHalfword => 2,
            Self::UnsignedWord | Self::Word => 4,
            Self::Doubleword => 8,
            Self::Capability => 16,
        }
    }
}

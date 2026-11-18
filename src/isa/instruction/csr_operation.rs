// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::xlen::Word;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum CsrOperation {
    ReadClear,
    ReadSet,
    ReadWrite,
}

impl CsrOperation {
    pub(crate) fn apply<W: Word>(self, previous: W, source_value: W) -> W {
        match self {
            Self::ReadWrite => source_value,
            Self::ReadSet => previous | source_value,
            Self::ReadClear => previous & !source_value,
        }
    }

    pub(crate) fn writes(self, source: u8) -> bool {
        self == Self::ReadWrite || source != 0
    }
}

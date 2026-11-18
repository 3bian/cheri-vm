// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Access {
    Load,
    Store,
    StoreCapability,
}

impl Access {
    pub(crate) fn writes(self) -> bool {
        !matches!(self, Self::Load)
    }
}

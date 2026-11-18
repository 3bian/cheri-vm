// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegisterPair(pub(super) u8);

impl RegisterPair {
    pub const fn destination(self) -> u8 {
        self.0 & 15
    }

    pub const fn first(self) -> u8 {
        self.0 >> 4
    }
}

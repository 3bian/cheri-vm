// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::pointer::rvy::Capability128;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Metadata {
    pub bits: u64,
    pub tag: bool,
}

impl Metadata {
    pub const fn at(self, address: u64) -> Capability128 {
        Capability128 {
            address,
            metadata: self.bits,
            tag: self.tag,
        }
    }

    pub const fn of(capability: Capability128) -> Self {
        Self {
            bits: capability.metadata,
            tag: capability.tag,
        }
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{Capability128, permissions};
use crate::isa::pointer::Violation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Window {
    base: u64,
    open: bool,
    refusal: Violation,
    span: u64,
}

impl Window {
    #[inline]
    pub fn admits(self, address: u64) -> Result<(), Violation> {
        if self.covers(address) {
            Ok(())
        } else {
            Err(self.refusal)
        }
    }

    #[inline]
    pub fn covers(self, address: u64) -> bool {
        self.open && address.wrapping_sub(self.base) <= self.span
    }

    pub fn of(capability: Capability128) -> Self {
        let (base, top) = capability.bounds();
        let length = top.saturating_sub(u128::from(base));
        let refusal = if !capability.tag {
            Some(Violation::Tag)
        } else if capability.sealed() {
            Some(Violation::Seal)
        } else if capability.permissions() & permissions::EXECUTE == 0 {
            Some(Violation::Permission)
        } else if length < 2 {
            Some(Violation::Bounds)
        } else {
            None
        };
        Self {
            base,
            open: refusal.is_none(),
            refusal: refusal.unwrap_or(Violation::Bounds),
            span: u64::try_from(length.saturating_sub(2)).unwrap_or(u64::MAX),
        }
    }
}

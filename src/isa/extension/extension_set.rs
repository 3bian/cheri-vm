// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{EXTENSIONS, named};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExtensionSet(pub(super) u32);

impl ExtensionSet {
    pub const A: Self = Self(1 << 7);
    pub const C: Self = Self(1 << 0);
    pub const CHERIOT: Self = Self(1 << 1);
    pub const D: Self = Self(1 << 11);
    pub const E: Self = Self(1 << 2);
    pub const F: Self = Self(1 << 10);
    pub const I: Self = Self(1 << 3);
    pub const M: Self = Self(1 << 4);
    pub const S: Self = Self(1 << 8);
    pub const SVYRG: Self = Self(1 << 21);
    pub const U: Self = Self(1 << 9);
    pub const Y: Self = Self(1 << 19);
    pub const ZBA: Self = Self(1 << 12);
    pub const ZBB: Self = Self(1 << 13);
    pub const ZBC: Self = Self(1 << 14);
    pub const ZBS: Self = Self(1 << 15);
    pub const ZCB: Self = Self(1 << 16);
    pub const ZICOND: Self = Self(1 << 17);
    pub const ZICSR: Self = Self(1 << 5);
    pub const ZIFENCEI: Self = Self(1 << 6);
    pub const ZKN: Self = Self(1 << 18);
    pub const ZYLG: Self = Self(1 << 20);

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn instruction_alignment(self) -> u32 {
        if self.contains(Self::C) { 2 } else { 4 }
    }

    pub fn named<'a>(
        names: impl IntoIterator<Item = &'a str>,
    ) -> Result<Self, &'a str> {
        names.into_iter().try_fold(Self::default(), |set, name| {
            match named(name) {
                Some(extension) if !set.contains(extension.set) => {
                    Ok(set.with(extension.set))
                }
                _ => Err(name),
            }
        })
    }

    pub fn names(self) -> impl Iterator<Item = &'static str> {
        EXTENSIONS
            .iter()
            .filter(move |extension| self.contains(extension.set))
            .map(|extension| extension.name)
    }

    pub const fn register_count(self) -> usize {
        if self.contains(Self::E) { 16 } else { 32 }
    }

    pub const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

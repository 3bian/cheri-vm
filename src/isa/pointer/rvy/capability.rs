// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::bounds::{self, EXPONENT_LIMIT};

pub mod permissions {
    pub const CAPABILITY: u64 = 1 << 5;
    pub const EXECUTE: u64 = 1 << 17;
    pub const GLOBAL: u64 = 1 << 4;
    pub const LOAD_GLOBAL: u64 = 1 << 2;
    pub const LOAD_MUTABLE: u64 = 1 << 1;
    pub const READ: u64 = 1 << 18;
    pub const SOFTWARE: u64 = 0xf << 6;
    pub const STORE_LOCAL: u64 = 1 << 3;
    pub const SYSTEM: u64 = 1 << 16;
    pub const WRITE: u64 = 1;

    pub const ALL: u64 =
        CAPABILITY | EXECUTE | LOAD_MUTABLE | READ | SOFTWARE | SYSTEM | WRITE;
    pub const LEVELS: u64 = GLOBAL | LOAD_GLOBAL | STORE_LOCAL;
    pub const RESERVED_ONES: u64 = 0x00f8_fc00;
}

const ARCHITECTURAL: u64 = 0xff << 45;
const GLOBAL: u64 = 1 << 43;
const SEALED: u64 = 1 << 27;
const SOFTWARE: u64 = 0xf << 60;
const USED: u64 = ARCHITECTURAL | GLOBAL | SEALED | SOFTWARE | bounds::FIELDS;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Capability128 {
    pub address: u64,
    pub metadata: u64,
    pub tag: bool,
}

impl Capability128 {
    pub const INFINITE: Self = Self {
        address: 0,
        metadata: (0x3f << 45) | SOFTWARE,
        tag: true,
    };

    pub fn alignment_mask(length: u64) -> u64 {
        let (fields, _) = bounds::encoded(0, length);
        match bounds::decoded(0, fields) {
            Some(decoded) if decoded.internal => {
                u64::MAX << (decoded.exponent + 3).min(63)
            }
            _ => u64::MAX,
        }
    }

    pub fn bounds(self) -> (u64, u128) {
        match bounds::decoded(self.address, self.metadata) {
            Some(decoded) if self.sound_fields() => (decoded.base, decoded.top),
            _ => (0, 0),
        }
    }

    pub fn contains(self, address: u64, width: u64) -> bool {
        let (base, top) = self.bounds();
        address >= base && u128::from(address) + u128::from(width) <= top
    }

    pub fn has_infinite_bounds(self) -> bool {
        bounds::decoded(self.address, self.metadata).is_some_and(|decoded| {
            decoded.internal && decoded.exponent == EXPONENT_LIMIT
        })
    }

    fn held(self) -> u64 {
        let held = self.metadata >> 45;
        let bit =
            |from: u32, to: u64| if held >> from & 1 == 1 { to } else { 0 };
        bit(0, permissions::CAPABILITY)
            | bit(1, permissions::WRITE)
            | bit(2, permissions::READ)
            | bit(3, permissions::EXECUTE)
            | bit(4, permissions::SYSTEM)
            | bit(5, permissions::LOAD_MUTABLE)
            | bit(6, permissions::LOAD_GLOBAL)
            | bit(7, permissions::STORE_LOCAL)
            | if self.metadata & GLOBAL != 0 {
                permissions::GLOBAL
            } else {
                0
            }
            | ((self.metadata >> 60) << 6)
            | permissions::RESERVED_ONES
    }

    pub const fn infinite(levels: bool) -> Self {
        if levels {
            Self {
                metadata: Self::INFINITE.metadata | (3 << 51) | GLOBAL,
                ..Self::INFINITE
            }
        } else {
            Self::INFINITE
        }
    }

    pub const fn integer(address: u64) -> Self {
        Self {
            address,
            metadata: 0,
            tag: false,
        }
    }

    pub fn is_sound(self) -> bool {
        self.sound_fields()
            && bounds::decoded(self.address, self.metadata).is_some()
    }

    pub fn is_subset_of(self, other: Self) -> bool {
        let (base, top) = self.bounds();
        let (other_base, other_top) = other.bounds();
        self.is_sound()
            && other.is_sound()
            && self.permissions() & !other.permissions() == 0
            && base >= other_base
            && top <= other_top
    }

    pub fn permissions(self) -> u64 {
        if !self.is_sound() {
            return permissions::RESERVED_ONES;
        }
        self.held()
    }

    pub fn reach(self) -> (u64, u64, u128) {
        match bounds::decoded(self.address, self.metadata) {
            Some(decoded) if self.sound_fields() => {
                (self.held(), decoded.base, decoded.top)
            }
            _ => (permissions::RESERVED_ONES, 0, 0),
        }
    }

    pub fn sealed(self) -> bool {
        self.metadata & SEALED != 0
    }

    fn sound_fields(self) -> bool {
        let held = self.metadata >> 45;
        let has = |bit: u32| held >> bit & 1 == 1;
        self.metadata & !USED == 0
            && (!has(0) || has(1) || has(2))
            && (!has(5) || has(0) && has(2))
            && (!has(6) || has(0) && has(2))
            && (!has(7) || has(0) && has(1))
            && (!has(4) || has(3))
    }

    pub fn with_address(self, address: u64) -> Self {
        let moved = Self { address, ..self };
        let kept = || match bounds::decoded(self.address, self.metadata) {
            Some(decoded) if self.sound_fields() => {
                (address >= decoded.base && u128::from(address) < decoded.top)
                    || moved.bounds() == (decoded.base, decoded.top)
            }
            _ => false,
        };
        Self {
            tag: self.tag && !self.sealed() && kept(),
            ..moved
        }
    }

    pub fn with_bounds(self, length: u64, exact: bool) -> Self {
        let (fields, precise) = bounds::encoded(self.address, length);
        let narrowed = Self {
            metadata: (self.metadata & !bounds::FIELDS) | fields,
            ..self
        };
        let (base, top) = narrowed.bounds();
        let (old_base, old_top) = self.bounds();
        Self {
            tag: self.tag
                && !self.sealed()
                && self.is_sound()
                && narrowed.is_sound()
                && u128::from(self.address) + u128::from(length) <= old_top
                && (precise || !exact)
                && base >= old_base
                && top <= old_top,
            ..narrowed
        }
    }

    pub fn with_seal(self, sealed: bool) -> Self {
        Self {
            metadata: if sealed {
                self.metadata | SEALED
            } else {
                self.metadata & !SEALED
            },
            ..self
        }
    }

    pub fn without_permissions(self, mask: u64) -> Self {
        let cleared = self.metadata & !(ARCHITECTURAL | GLOBAL | SOFTWARE);
        if !self.is_sound() {
            return Self {
                metadata: cleared,
                tag: false,
                ..self
            };
        }
        let kept = self.permissions() & !mask;
        let has = |permission: u64| kept & permission != 0;
        let capability = has(permissions::CAPABILITY)
            && (has(permissions::READ) || has(permissions::WRITE));
        let reads = capability && has(permissions::READ);
        let writes = capability && has(permissions::WRITE);
        let held = u64::from(capability)
            | u64::from(has(permissions::WRITE)) << 1
            | u64::from(has(permissions::READ)) << 2
            | u64::from(has(permissions::EXECUTE)) << 3
            | u64::from(has(permissions::SYSTEM) && has(permissions::EXECUTE))
                << 4
            | u64::from(has(permissions::LOAD_MUTABLE) && reads) << 5
            | u64::from(has(permissions::LOAD_GLOBAL) && reads) << 6
            | u64::from(has(permissions::STORE_LOCAL) && writes) << 7;
        let global = if has(permissions::GLOBAL) { GLOBAL } else { 0 };
        let metadata = cleared
            | held << 45
            | global
            | ((kept & permissions::SOFTWARE) >> 6) << 60;
        Self {
            metadata,
            tag: self.tag
                && !(self.sealed()
                    && (metadata ^ self.metadata) & !GLOBAL != 0),
            ..self
        }
    }
}

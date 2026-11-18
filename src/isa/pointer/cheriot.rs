// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub(crate) mod capability;

pub use capability::{Capability, permissions};

use super::{Access, PointerModel, Violation};

pub enum Cheriot {}

impl PointerModel for Cheriot {
    type Address = u32;
    type Fetch = Capability;
    type Register = Capability;

    fn address(register: Capability) -> u32 {
        register.address()
    }

    fn check_access(
        authority: Capability,
        address: u32,
        width: usize,
        access: Access,
    ) -> Result<(), Violation> {
        if !authority.tag() {
            return Err(Violation::Tag);
        }
        if authority.sealed() {
            return Err(Violation::Seal);
        }
        let held = authority.permissions();
        let needed = if access.writes() {
            permissions::STORE
        } else {
            permissions::LOAD
        };
        if held & needed == 0 {
            return Err(Violation::Permission);
        }
        if access == Access::StoreCapability
            && held & permissions::LOAD_STORE_CAPABILITY == 0
        {
            return Err(Violation::CapabilityPermission);
        }
        if !authority.contains(address, width as u64) {
            return Err(Violation::Bounds);
        }
        if width == 8 && !address.is_multiple_of(8) {
            return Err(Violation::Alignment);
        }
        Ok(())
    }

    #[inline]
    fn check_fetch(
        authority: Capability,
        address: u32,
    ) -> Result<(), Violation> {
        if !authority.tag() {
            return Err(Violation::Tag);
        }
        if authority.sealed() {
            return Err(Violation::Seal);
        }
        if authority.permissions() & permissions::EXECUTE == 0 {
            return Err(Violation::Permission);
        }
        let (base, top) = authority.bounds();
        if address < base || u64::from(address) + 2 > top {
            return Err(Violation::Bounds);
        }
        Ok(())
    }
}

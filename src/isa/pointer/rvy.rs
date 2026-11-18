// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod bounds;
pub(crate) mod capability;
mod window;

pub use capability::{Capability128, permissions};
pub use window::Window;

use super::{Access, PointerModel, Violation};

pub enum Rvy {}

impl PointerModel for Rvy {
    type Address = u64;
    type Fetch = Window;
    type Register = Capability128;

    fn address(register: Capability128) -> u64 {
        register.address
    }

    fn check_access(
        authority: Capability128,
        address: u64,
        width: usize,
        access: Access,
    ) -> Result<(), Violation> {
        usable(authority)?;
        let (held, base, top) = authority.reach();
        let needed = if access.writes() {
            permissions::WRITE
        } else {
            permissions::READ
        };
        if held & needed == 0 {
            return Err(Violation::Permission);
        }
        if address < base || u128::from(address) + width as u128 > top {
            return Err(Violation::Bounds);
        }
        Ok(())
    }

    #[inline]
    fn check_fetch(authority: Window, address: u64) -> Result<(), Violation> {
        authority.admits(address)
    }
}

fn usable(authority: Capability128) -> Result<(), Violation> {
    if !authority.tag {
        return Err(Violation::Tag);
    }
    if authority.sealed() {
        return Err(Violation::Seal);
    }
    Ok(())
}

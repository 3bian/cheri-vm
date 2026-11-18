// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::xlen::Word;
use super::{Access, Violation};

pub trait PointerModel {
    type Address: Word;
    type Fetch: Copy;
    type Register: Copy;

    fn address(register: Self::Register) -> Self::Address;

    fn check_access(
        authority: Self::Register,
        address: Self::Address,
        width: usize,
        access: Access,
    ) -> Result<(), Violation>;

    fn check_fetch(
        authority: Self::Fetch,
        address: Self::Address,
    ) -> Result<(), Violation>;
}

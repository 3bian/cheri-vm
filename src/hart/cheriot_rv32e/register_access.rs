// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::pointer::cheriot::Capability;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegisterAccess {
    Read { index: u8, value: Capability },
    Write { index: u8, value: Capability },
}

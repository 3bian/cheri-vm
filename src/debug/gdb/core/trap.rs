// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub(crate) struct Trap<F> {
    pub address: u64,
    pub cause: u32,
    pub fault: F,
    pub interrupt: bool,
}

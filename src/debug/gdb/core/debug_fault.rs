// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DebugFault<C> {
    pub address: u64,
    pub cause: u32,
    pub context: C,
    pub interrupt: bool,
    pub resulting_pc: u64,
    pub value: u64,
}

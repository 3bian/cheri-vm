// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Trapped {
    pub address: u64,
    pub cause: u32,
    pub interrupt: bool,
}

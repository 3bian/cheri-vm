// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Xlen;
use core::fmt::Debug;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rv64 {}

impl Xlen for Rv64 {
    const BITS: u32 = 64;
    type Word = u64;
}

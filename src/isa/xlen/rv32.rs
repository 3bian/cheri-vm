// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Xlen;
use core::fmt::Debug;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rv32 {}

impl Xlen for Rv32 {
    const BITS: u32 = 32;
    type Word = u32;
}

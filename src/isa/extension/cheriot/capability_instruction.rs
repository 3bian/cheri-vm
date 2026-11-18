// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Operation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilityInstruction {
    pub destination: u8,
    pub first: u8,
    pub immediate: i32,
    pub operation: Operation,
    pub second: u8,
}

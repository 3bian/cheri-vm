// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod atomic_width;

pub use atomic_width::AtomicWidth;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtomicOperation {
    Add,
    And,
    LoadReserved,
    Maximum,
    MaximumUnsigned,
    Minimum,
    MinimumUnsigned,
    Or,
    StoreConditional,
    Swap,
    Xor,
}

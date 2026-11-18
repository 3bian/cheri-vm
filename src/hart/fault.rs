// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod exception;

pub use exception::Exception;

pub(crate) const INTERRUPT: u32 = 1 << 31;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Fault<W = u32> {
    pub cause: u32,
    pub value: W,
}

impl Fault {
    pub(crate) fn illegal() -> Self {
        Self { cause: 2, value: 0 }
    }
}

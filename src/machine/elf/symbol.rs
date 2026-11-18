// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::SYMBOL_FUNCTION;

pub struct Symbol<'a> {
    pub kind: u8,
    pub name: &'a str,
    pub section: u16,
    pub size: u64,
    pub value: u64,
}

impl Symbol<'_> {
    pub fn is_function(&self) -> bool {
        self.kind == SYMBOL_FUNCTION && self.size > 0
    }
}

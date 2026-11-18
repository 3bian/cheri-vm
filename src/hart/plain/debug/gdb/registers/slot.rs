// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Slot {
    pub(super) bits: usize,
    pub(super) name: &'static str,
    pub(super) raw: usize,
    pub(super) value: Option<u64>,
}

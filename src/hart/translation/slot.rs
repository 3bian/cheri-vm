// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Default)]
pub(super) struct Slot {
    pub(super) flags: u64,
    pub(super) frame: u64,
    pub(super) tag: u64,
}

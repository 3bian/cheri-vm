// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub(super) struct Frame {
    pub(super) last: bool,
    pub(super) opcode: u8,
    pub(super) payload: Vec<u8>,
}

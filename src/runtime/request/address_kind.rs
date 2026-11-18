// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug)]
pub enum AddressKind {
    Breakpoint,
    Completion,
    Raw,
}

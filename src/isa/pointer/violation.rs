// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Violation {
    Alignment,
    Bounds,
    CapabilityPermission,
    Permission,
    Seal,
    Tag,
}

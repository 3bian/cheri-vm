// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub mod base;
pub mod csr;
pub mod draft;
pub mod encoding;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    Add,
    AddImmediate,
    AlignmentMask,
    Base,
    Build,
    ClearPermissions,
    Equal,
    High,
    Length,
    Move,
    Permissions,
    SealEntry,
    SetAddress,
    SetBounds,
    SetBoundsImmediate,
    SetBoundsRounded,
    SetHigh,
    ShiftAdd,
    ShiftAddUnsignedWord,
    Subset,
    Tag,
    Top,
    Type,
    Unseal,
}

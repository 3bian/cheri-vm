// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Operation;

pub const ADD_IMMEDIATE: u32 = 4;
pub const ATOMIC: u32 = 3;
pub const ATOMIC_LOAD_RESERVED: u32 = 2;
pub const ATOMIC_STORE_CONDITIONAL: u32 = 3;
pub const ATOMIC_SWAP: u32 = 1;
pub const COMPRESSED_LOAD: u16 = 1;
pub const COMPRESSED_STORE: u16 = 5;
pub const LOAD: u32 = 1;
pub const MAJOR: u32 = 0x7b;
pub const MASKS: u32 = 0x78;
pub const MISCELLANEOUS: u32 = 5;
pub const QUERIES: u32 = 0x7a;

pub const QUERY: [(u32, Operation); 7] = [
    (0, Operation::Base),
    (1, Operation::Permissions),
    (2, Operation::Top),
    (3, Operation::Length),
    (4, Operation::Tag),
    (5, Operation::Type),
    (7, Operation::High),
];

pub const REGISTER: u32 = 0;
pub const SET_BOUNDS_IMMEDIATE: u32 = 7;

pub const SHIFT_ADD: [(u32, Operation, i16); 8] = [
    (0x05, Operation::ShiftAdd, 1),
    (0x0d, Operation::ShiftAdd, 2),
    (0x15, Operation::ShiftAdd, 3),
    (0x1d, Operation::ShiftAdd, 4),
    (0x25, Operation::ShiftAddUnsignedWord, 1),
    (0x2d, Operation::ShiftAddUnsignedWord, 2),
    (0x35, Operation::ShiftAddUnsignedWord, 3),
    (0x3d, Operation::ShiftAddUnsignedWord, 4),
];

pub const STORE: u32 = 2;

pub const TWO_REGISTER: [(u32, Operation); 11] = [
    (0x01, Operation::SetHigh),
    (0x03, Operation::Add),
    (0x06, Operation::Equal),
    (0x07, Operation::Unseal),
    (0x0b, Operation::SetAddress),
    (0x0e, Operation::Subset),
    (0x0f, Operation::Build),
    (0x13, Operation::ClearPermissions),
    (0x17, Operation::SealEntry),
    (0x1b, Operation::SetBounds),
    (0x23, Operation::SetBoundsRounded),
];

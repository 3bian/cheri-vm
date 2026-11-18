// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub struct Segment {
    pub alignment: u64,
    pub file_size: u64,
    pub flags: u32,
    pub kind: u32,
    pub memory_size: u64,
    pub offset: u64,
    pub physical_address: u64,
    pub virtual_address: u64,
}

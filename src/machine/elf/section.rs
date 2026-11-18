// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{
    SECTION_COMPRESSED, SECTION_EXECUTABLE, SECTION_NO_BITS,
    SECTION_PROGRAM_BITS,
};

pub struct Section<'a> {
    pub address: u64,
    pub(super) bytes: &'a [u8],
    pub entry_size: u64,
    pub flags: u64,
    pub kind: u32,
    pub link: u32,
    pub name: &'a str,
    pub(super) name_offset: usize,
    pub offset: u64,
    pub size: u64,
}

impl<'a> Section<'a> {
    pub fn data(&self) -> Option<&'a [u8]> {
        if self.kind == SECTION_NO_BITS {
            return Some(&[]);
        }
        let start = usize::try_from(self.offset).ok()?;
        let end = start.checked_add(usize::try_from(self.size).ok()?)?;
        self.bytes.get(start..end)
    }

    pub fn is_code(&self) -> bool {
        self.kind == SECTION_PROGRAM_BITS
            && self.flags & SECTION_EXECUTABLE != 0
    }

    pub fn is_compressed(&self) -> bool {
        self.flags & SECTION_COMPRESSED != 0
            || self.name.starts_with(".zdebug_")
    }
}

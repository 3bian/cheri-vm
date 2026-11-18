// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::memory::tag_bitmap::*;

impl TagBitmap {
    pub(crate) fn storage_bytes(&self) -> usize {
        self.bytes.len()
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::tag_bitmap::TagBitmap;

pub(crate) struct CheriotTags {
    pub(crate) bitmap: TagBitmap,
    pub(crate) granule: usize,
}

impl CheriotTags {
    pub(crate) const GRANULE_BYTES: usize = 8;

    pub(crate) fn clear(&mut self, offset: usize, length: usize) {
        if length == 0 {
            return;
        }

        let first = offset / self.granule;
        let last = (offset + length - 1) / self.granule;
        self.bitmap.clear(first, last);
    }

    pub(crate) fn get(&self, offset: usize) -> bool {
        self.bitmap.get(offset / self.granule)
    }

    pub(crate) fn new(memory_size: usize) -> Self {
        Self::with_granule(memory_size, Self::GRANULE_BYTES)
    }

    pub(crate) fn set(&mut self, offset: usize, tag: bool) {
        self.bitmap.set(offset / self.granule, tag);
    }

    pub(crate) fn with_granule(memory_size: usize, granule: usize) -> Self {
        Self {
            bitmap: TagBitmap::new(memory_size / granule),
            granule,
        }
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::memory::cheriot_tags::*;

impl CheriotTags {
    pub(crate) fn storage_bytes(&self) -> usize {
        self.bitmap.storage_bytes()
    }
}

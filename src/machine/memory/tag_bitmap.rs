// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub(crate) struct TagBitmap {
    pub(crate) bytes: Vec<u8>,
}

impl TagBitmap {
    pub(crate) fn clear(&mut self, first: usize, last: usize) {
        let first_byte = first / 8;
        let last_byte = last / 8;
        let first_mask = u8::MAX << (first % 8);
        let last_mask = u8::MAX >> (7 - last % 8);

        if first_byte == last_byte {
            self.bytes[first_byte] &= !(first_mask & last_mask);
        } else {
            self.bytes[first_byte] &= !first_mask;
            self.bytes[first_byte + 1..last_byte].fill(0);
            self.bytes[last_byte] &= !last_mask;
        }
    }

    pub(crate) fn get(&self, index: usize) -> bool {
        self.bytes[index / 8] & (1 << (index % 8)) != 0
    }

    pub(crate) fn new(tags: usize) -> Self {
        Self {
            bytes: vec![0; tags.div_ceil(8)],
        }
    }

    pub(crate) fn set(&mut self, index: usize, tag: bool) {
        let mask = 1 << (index % 8);
        let byte = &mut self.bytes[index / 8];

        if tag {
            *byte |= mask;
        } else {
            *byte &= !mask;
        }
    }
}

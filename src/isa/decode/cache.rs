// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::DecodedInstruction;

const ENTRIES: usize = 4096;

pub(crate) struct DecodeCache {
    entries: Box<[Option<DecodedInstruction>]>,
}

impl DecodeCache {
    #[inline]
    pub(crate) fn decode(
        &mut self,
        encoded: u32,
        decode: impl FnOnce(u32) -> Option<DecodedInstruction>,
    ) -> Option<DecodedInstruction> {
        let slot = &mut self.entries[slot(encoded)];
        if let Some(cached) = slot
            && cached.encoded == encoded
        {
            return Some(*cached);
        }
        let decoded = decode(encoded)?;
        if decoded.encoded == encoded {
            *slot = Some(decoded);
        }
        Some(decoded)
    }

    pub(crate) fn new() -> Self {
        Self {
            entries: vec![None; ENTRIES].into_boxed_slice(),
        }
    }
}

impl Default for DecodeCache {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn slot(encoded: u32) -> usize {
    ((encoded ^ (encoded >> 12) ^ (encoded >> 22)) as usize) & (ENTRIES - 1)
}

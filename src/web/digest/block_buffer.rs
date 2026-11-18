// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub(super) const BLOCK_LENGTH: usize = 64;
const LENGTH_FIELD: usize = 8;
const PADDING_START: u8 = 0x80;

pub(super) type Block = [u8; BLOCK_LENGTH];

pub(super) struct BlockBuffer {
    filled: usize,
    message_length: u64,
    pending: Block,
}

impl BlockBuffer {
    pub(super) fn finish(mut self, mut compress: impl FnMut(&Block)) {
        let bits = self.message_length.wrapping_mul(8);
        self.pending[self.filled] = PADDING_START;
        self.pending[self.filled + 1..].fill(0);
        if self.filled + 1 > BLOCK_LENGTH - LENGTH_FIELD {
            compress(&self.pending);
            self.pending.fill(0);
        }
        self.pending[BLOCK_LENGTH - LENGTH_FIELD..]
            .copy_from_slice(&bits.to_be_bytes());
        compress(&self.pending);
    }

    pub(super) fn new() -> Self {
        Self {
            pending: [0; BLOCK_LENGTH],
            filled: 0,
            message_length: 0,
        }
    }

    pub(super) fn update(
        &mut self,
        mut data: &[u8],
        mut compress: impl FnMut(&Block),
    ) {
        self.message_length =
            self.message_length.wrapping_add(data.len() as u64);
        if self.filled > 0 {
            let taken = (BLOCK_LENGTH - self.filled).min(data.len());
            self.pending[self.filled..self.filled + taken]
                .copy_from_slice(&data[..taken]);
            self.filled += taken;
            data = &data[taken..];
            if self.filled < BLOCK_LENGTH {
                return;
            }
            compress(&self.pending);
            self.filled = 0;
        }
        let (blocks, rest) = data.as_chunks::<BLOCK_LENGTH>();
        for block in blocks {
            compress(block);
        }
        self.pending[..rest.len()].copy_from_slice(rest);
        self.filled = rest.len();
    }
}

pub(super) fn bytes<const WORDS: usize, const LENGTH: usize>(
    state: [u32; WORDS],
) -> [u8; LENGTH] {
    let mut digest = [0; LENGTH];
    for (bytes, word) in digest.as_chunks_mut::<4>().0.iter_mut().zip(state) {
        *bytes = word.to_be_bytes();
    }
    digest
}

pub(super) fn words<const COUNT: usize>(block: &Block) -> [u32; COUNT] {
    let mut schedule = [0; COUNT];
    for (word, bytes) in schedule.iter_mut().zip(block.as_chunks::<4>().0) {
        *word = u32::from_be_bytes(*bytes);
    }
    schedule
}

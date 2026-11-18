// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::block_buffer::{Block, BlockBuffer, bytes, words};

const DIGEST_LENGTH: usize = 20;

const INITIAL_STATE: [u32; 5] =
    [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];

const ROUNDS: usize = 80;
const ROUNDS_PER_STAGE: usize = 20;

const STAGE_CONSTANTS: [u32; 4] =
    [0x5a827999, 0x6ed9eba1, 0x8f1bbcdc, 0xca62c1d6];

pub struct Sha1 {
    buffer: BlockBuffer,
    state: [u32; 5],
}

impl Sha1 {
    pub fn finish(mut self) -> [u8; DIGEST_LENGTH] {
        let state = &mut self.state;
        self.buffer.finish(|block| compress(state, block));
        bytes(self.state)
    }

    pub fn new() -> Self {
        Self {
            state: INITIAL_STATE,
            buffer: BlockBuffer::new(),
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        let state = &mut self.state;
        self.buffer.update(data, |block| compress(state, block));
    }
}

impl Default for Sha1 {
    fn default() -> Self {
        Self::new()
    }
}

fn compress(state: &mut [u32; 5], block: &Block) {
    let mut schedule: [u32; ROUNDS] = words(block);
    for index in 16..ROUNDS {
        schedule[index] = (schedule[index - 3]
            ^ schedule[index - 8]
            ^ schedule[index - 14]
            ^ schedule[index - 16])
            .rotate_left(1);
    }
    let [mut a, mut b, mut c, mut d, mut e] = *state;
    for (index, word) in schedule.iter().enumerate() {
        let stage = index / ROUNDS_PER_STAGE;
        let mixed = match stage {
            0 => (b & c) | (!b & d),
            2 => (b & c) | (b & d) | (c & d),
            _ => b ^ c ^ d,
        };
        let next = a
            .rotate_left(5)
            .wrapping_add(mixed)
            .wrapping_add(e)
            .wrapping_add(STAGE_CONSTANTS[stage])
            .wrapping_add(*word);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = next;
    }
    for (value, added) in state.iter_mut().zip([a, b, c, d, e]) {
        *value = value.wrapping_add(added);
    }
}

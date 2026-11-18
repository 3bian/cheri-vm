// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod bursts;

pub(crate) use bursts::Bursts;

struct Burst {
    bytes: Vec<u8>,
    mode: u8,
}

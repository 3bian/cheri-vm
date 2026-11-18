// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod block_buffer;
mod sha1;

pub use sha1::Sha1;

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn sha1(data: &[u8]) -> [u8; 20] {
    let mut hash = Sha1::new();
    hash.update(data);
    hash.finish()
}

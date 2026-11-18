// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod rv32;
mod rv64;
mod word;

pub use rv32::Rv32;
pub use rv64::Rv64;
pub use word::Word;

pub trait Xlen: 'static {
    const BITS: u32;
    type Word: Word;
}

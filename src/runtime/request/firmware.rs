// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

#[derive(Debug)]
pub enum Firmware {
    Elf(PathBuf),
    Raw { path: PathBuf, address: u64 },
}

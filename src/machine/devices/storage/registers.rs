// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

registers! {
    Register {
        Sector = 0, "sector";
        WordIndex = 4, "wordIndex";
        Data = 8, "data";
        Command = 12, "command";
        Status = 16, "status";
        Control = 20, "control";
    }
}

pub(crate) const COMMAND_READ: u32 = 1;
pub(crate) const COMMAND_WRITE: u32 = 2;
pub(crate) const CONTROL_INTERRUPT_ENABLE: u32 = 1 << 0;
pub(crate) const SECTOR_BYTES: usize = 512;
pub(crate) const SECTOR_WORDS: u32 = 128;
pub(crate) const STATUS_COMPLETE: u32 = 1 << 0;
pub(crate) const STATUS_ERROR: u32 = 1 << 1;
pub(crate) const STATUS_PRESENT: u32 = 1 << 2;

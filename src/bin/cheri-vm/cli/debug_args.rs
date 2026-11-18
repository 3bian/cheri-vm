// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{DAP, GDB, GDB_PORT, RunArgs, port};
use cheri_vm::program::arguments::Parsed;
use std::path::PathBuf;

pub(crate) struct DebugArgs {
    pub dap: bool,
    pub gdb: Option<PathBuf>,
    pub gdb_port: u16,
    pub run: RunArgs,
}

impl DebugArgs {
    pub(crate) fn from(parsed: &Parsed) -> Self {
        Self {
            dap: parsed.has(DAP.long),
            gdb: parsed.one(GDB.long).map(PathBuf::from),
            gdb_port: port(parsed, GDB_PORT.long),
            run: RunArgs::from(parsed),
        }
    }
}

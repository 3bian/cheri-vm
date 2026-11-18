// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::{cli::DebugArgs, error::Result};
use cheri_vm::program::command::Context;

pub(crate) mod dap;
mod start;

pub(crate) fn handle(arguments: DebugArgs, context: &Context) -> Result<u8> {
    start::handle(arguments, context)
}

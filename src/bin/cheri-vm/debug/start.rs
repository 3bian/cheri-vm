// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::cli::DebugArgs;
use crate::error::{AppError, Result};
use cheri_vm::program::command::Context;

pub(super) fn handle(d: DebugArgs, context: &Context) -> Result<u8> {
    if d.dap {
        return super::dap::serve(d.run);
    }
    if d.gdb_port == 0 {
        return Err(AppError::Message("--gdb-port must be nonzero".into()));
    }
    let port = d.gdb_port;
    let mut request = crate::run::request(d.run)?;
    request.debugger = Some(std::net::SocketAddr::new(
        request.peripherals.web.host,
        port,
    ));
    request.debugger_client = d.gdb;
    cheri_vm::runtime::debug_request(request, &|address| {
        context.shell.status(
            "Listening",
            &format!("on {}:{} for a GDB client", address.ip(), address.port()),
        )
    })
    .map_err(|e| AppError::Message(e.to_string()))?;
    Ok(0)
}

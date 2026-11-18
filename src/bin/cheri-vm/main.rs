// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod cli;
mod debug;
mod error;
mod profile;
mod run;

use cheri_vm::program::arguments::Parsed;
use cheri_vm::program::command::Context;
use cheri_vm::program::shell::Shell;
use std::process::ExitCode;

#[cfg(test)]
#[path = "../../../tests/binary/binary.rs"]
mod binary_tests;

#[cfg(test)]
#[path = "../../../tests/support/support.rs"]
mod support;

fn dispatch(parsed: &Parsed, shell: Shell) -> error::Result<u8> {
    let context = Context { shell };
    match parsed.command.map(|command| command.name) {
        Some("debug") => debug::handle(cli::DebugArgs::from(parsed), &context),
        Some("profile") => cheri_vm::program::command::run(
            cli::ProfileArgs::from(parsed),
            &context,
        ),
        _ => run::handle(cli::RunCommandArgs::from(parsed), &context),
    }
}

fn main() -> ExitCode {
    cheri_vm::program::start::run(
        &cli::PROGRAM,
        cheri_vm::program::settings::color,
        dispatch,
    )
}

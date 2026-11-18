// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#![allow(dead_code, unused_imports, unreachable_pub)]

mod commands_page;
mod documented;
mod help;
mod programs;

pub use commands_page::assert_commands_page;
pub use documented::{PROGRAM, documented_commands};
pub use help::{
    assert_readable, assert_snapshot, assert_version, every_command, help,
    listed_commands,
};
pub use programs::{
    Server, VM, alone, application, finish, fixture_file, flat, repository,
    run_application, stderr, stdout, vm,
};

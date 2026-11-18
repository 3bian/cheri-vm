// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub mod arguments;
pub mod command;
pub mod commands;
pub mod diagnostic;
pub mod help;
pub mod options;
pub mod output;
pub mod report;
pub mod settings;
pub mod shell;
pub mod start;
pub mod temporary;
pub mod wording;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

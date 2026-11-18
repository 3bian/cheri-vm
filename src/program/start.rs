// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::arguments::{Outcome, Parsed, Program};
use super::shell::Shell;
use crate::program::output::Color;
use std::process::ExitCode;

pub const ASKED_TO_END: i32 = 143;
pub const USAGE_ERROR: u8 = 2;

fn chosen_color(setting: fn() -> Color) -> Color {
    let arguments: Vec<String> = std::env::args_os()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect();
    let named =
        arguments.iter().enumerate().find_map(
            |(index, argument)| match argument.strip_prefix("--color") {
                Some("") => arguments.get(index + 1).cloned(),
                Some(rest) => rest.strip_prefix('=').map(str::to_owned),
                None => None,
            },
        );
    named
        .and_then(|name| Color::named(&name))
        .unwrap_or_else(setting)
}

#[cfg(unix)]
fn end_when_asked() {
    use nix::sys::signal::{SigSet, Signal};
    let mut asked = SigSet::empty();
    asked.add(Signal::SIGTERM);
    if asked.thread_block().is_err() {
        return;
    }
    let waiter = std::thread::Builder::new().name("end".to_owned());
    let _ = waiter.spawn(move || {
        if asked.wait().is_ok() {
            std::process::exit(ASKED_TO_END);
        }
    });
}

#[cfg(not(unix))]
fn end_when_asked() {}

pub fn run<E: std::fmt::Display>(
    program: &'static Program,
    color: fn() -> Color,
    dispatch: impl FnOnce(&Parsed, Shell) -> Result<u8, E>,
) -> ExitCode {
    take_interrupts();
    end_when_asked();
    let shell = Shell::new(chosen_color(color));
    let parsed = match program.parse(std::env::args_os().skip(1)) {
        Outcome::Run(parsed) => parsed,
        Outcome::Help(command) => {
            shell.stdout(&super::help::render(program, command));
            return ExitCode::SUCCESS;
        }
        Outcome::Version => {
            shell.stdout(&format!("{} {}\n", program.name, program.version));
            return ExitCode::SUCCESS;
        }
        Outcome::Refused(refusal, command) => {
            let path = match command {
                Some(command) => format!("{} {}", program.name, command.name),
                None => program.name.to_owned(),
            };
            shell.error(&format!(
                "{refusal}\n\nRun `{path} --help` for the options."
            ));
            return ExitCode::from(USAGE_ERROR);
        }
    };
    match dispatch(&parsed, shell) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            shell.error(&error.to_string());
            ExitCode::FAILURE
        }
    }
}

#[cfg(unix)]
fn take_interrupts() {
    use nix::sys::signal::{SigSet, Signal};
    let mut held_by_a_parent = SigSet::empty();
    held_by_a_parent.add(Signal::SIGINT);
    held_by_a_parent.add(Signal::SIGQUIT);
    let _ = held_by_a_parent.thread_unblock();
}

#[cfg(not(unix))]
fn take_interrupts() {}

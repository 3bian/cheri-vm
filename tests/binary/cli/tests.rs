// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::cli::PROGRAM;
use crate::support::documented_commands;
use cheri_vm::program::arguments::Outcome;

#[test]
fn every_documented_command_line_of_this_program_parses() {
    let mut failures = Vec::new();
    let commands: Vec<_> = documented_commands()
        .into_iter()
        .filter(|command| command.words[0] == "cheri-vm")
        .collect();
    assert!(
        commands.len() >= 10,
        "only {} command lines were found",
        commands.len()
    );
    for command in commands {
        if let Err(error) = parses(&command.words) {
            failures.push(format!(
                "{}: {}: {error}",
                command.place(),
                command.words.join(" ")
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn parses(words: &[String]) -> Result<(), String> {
    match PROGRAM.parse(words.iter().skip(1).map(Into::into)) {
        Outcome::Refused(refusal, _) => Err(refusal),
        _ => Ok(()),
    }
}

#[test]
fn the_commands_page_lists_every_command_and_option_of_this_program() {
    crate::support::assert_commands_page(&PROGRAM);
}

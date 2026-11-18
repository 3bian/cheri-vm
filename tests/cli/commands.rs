// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::support::{
    VM, assert_readable, assert_snapshot, every_command, help, listed_commands,
};
use cheri_vm::program::commands::{COMMANDS, names};
use std::path::Path;

#[test]
fn every_command_has_a_help_snapshot() {
    for command in every_command(names(COMMANDS)) {
        let file = match command.first() {
            Some(command) => format!("cheri-vm/{command}.txt"),
            None => "cheri-vm.txt".to_owned(),
        };
        assert_snapshot(&snapshots(), &file, &help(VM, &command));
    }
}

#[test]
fn every_help_screen_fits_eighty_columns_and_reads_as_sentences() {
    for command in every_command(names(COMMANDS)) {
        assert_readable(VM, &command, &help(VM, &command));
    }
}

#[test]
fn it_has_exactly_the_commands_the_shared_list_names_with_their_summaries() {
    let listed = listed_commands(&help(VM, &[]));
    assert_eq!(listed, names(COMMANDS).collect::<Vec<_>>());
    for command in COMMANDS {
        let flat =
            |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
        let text = flat(&help(VM, &[command.name]));
        assert!(
            text.contains(&flat(command.summary)),
            "{}: {text}",
            command.name
        );
    }
}

fn snapshots() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
}

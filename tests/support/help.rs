// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::programs::{run_application, stdout};
use std::path::Path;

const UPDATE: &str = "CHERI_VM_TEST_UPDATE_SNAPSHOTS";

pub fn assert_readable(name: &str, command: &[&str], text: &str) {
    let lines: Vec<&str> = text.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        assert!(line.chars().count() <= 80, "{name} {command:?}: {line}");
        let ends_description = line.starts_with("    ")
            && lines
                .get(index + 1)
                .is_none_or(|next| !next.starts_with("    "));
        if ends_description {
            assert!(line.ends_with('.'), "{name} {command:?}: {line}");
        }
    }
}

pub fn assert_snapshot(folder: &Path, name: &str, actual: &str) {
    let path = folder.join(name);
    if update_snapshots() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("{name} is missing; run the tests with {UPDATE}=1 to write it")
    });
    assert_eq!(
        actual, expected,
        "{name} differs; run the tests with {UPDATE}=1 to update it"
    );
}

pub fn assert_version(name: &str) -> String {
    let mut printed = Vec::new();
    for option in ["--version", "-V"] {
        let output = run_application(name, &[option]);
        assert!(output.status.success(), "{name} {option}");
        let line = stdout(&output);
        let version = line.trim().strip_prefix(name).unwrap().trim().to_owned();
        printed.push(version);
    }
    assert_eq!(printed[0], printed[1], "{name}");
    printed.remove(0)
}

pub fn every_command<'a>(
    commands: impl Iterator<Item = &'a str>,
) -> Vec<Vec<&'a str>> {
    std::iter::once(Vec::new())
        .chain(commands.map(|command| vec![command]))
        .collect()
}

pub fn help(name: &str, command: &[&str]) -> String {
    let mut arguments = command.to_vec();
    arguments.push("--help");
    let printed = run_application(name, &arguments);
    assert!(printed.status.success(), "{name} {command:?}");
    stdout(&printed)
}

pub fn listed_commands(help: &str) -> Vec<String> {
    help.split("\nCommands:\n")
        .nth(1)
        .and_then(|rest| rest.split("\nOptions:\n").next())
        .unwrap_or_default()
        .lines()
        .filter(|line| line.starts_with("  ") && !line.starts_with("    "))
        .map(|line| line.trim().to_owned())
        .collect()
}

pub(super) fn update_snapshots() -> bool {
    std::env::var_os(UPDATE).is_some_and(|value| value == "1")
}

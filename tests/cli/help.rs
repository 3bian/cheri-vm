// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::support::{VM, assert_version, help, run_application, stdout};

#[test]
fn a_usage_error_exits_with_status_two() {
    let refused = crate::support::vm(&["run", "--raw", "a", "--profile", "b"]);
    assert_eq!(refused.status.code(), Some(2));
}

#[test]
fn it_prints_its_version() {
    assert_eq!(assert_version(VM), env!("CARGO_PKG_VERSION"));
}

#[test]
fn its_readme_shows_the_help_screen_as_the_program_prints_it() {
    let readme = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/README.md"
    ))
    .unwrap();
    let screen = include_str!("../snapshots/cheri-vm.txt");
    assert!(
        readme.contains(&format!("```text\n{screen}```")),
        "README.md does not show tests/snapshots/cheri-vm.txt"
    );
}

#[test]
fn run_and_debug_accept_the_same_firmware_options() {
    for command in ["run", "debug"] {
        let text = help(VM, &[command]);
        for option in
            ["--profile", "--raw", "--address", "--breakpoint", "--steps"]
        {
            assert!(text.contains(option), "{command} is missing {option}");
        }
        assert!(!text.contains("--build\n"), "{command}");
    }
}

#[test]
fn with_no_command_it_shows_its_help_screen() {
    let bare = run_application(VM, &[]);
    assert!(bare.status.success());
    assert_eq!(stdout(&bare), help(VM, &[]));
}

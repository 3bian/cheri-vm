// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::program::arguments::{
    CommandSpec, Flag, Outcome, Parsed, Positional, Program,
};
use std::ffi::OsString;

static PROGRAM: Program = Program {
    commands: &[
        CommandSpec {
            name: "show",
            options: &[
                Flag::switch("all", "Show everything").shortened('a'),
                Flag::choice("form", &["short", "long"], "How to show it"),
                Flag::value("name", "NAME", "What to show").repeated(),
                Flag::number("count", "N", 9, "How many")
                    .with_default("3")
                    .requiring(&["all"]),
                Flag::switch("quiet", "Say nothing").conflicting(&["all"]),
            ],
            positionals: &[Positional {
                help: "File to read",
                name: "FILE",
                unless: None,
            }],
            summary: "Show a thing",
        },
        CommandSpec {
            name: "open",
            options: &[Flag::switch("all", "Open everything")],
            positionals: &[Positional {
                help: "File to open",
                name: "FILE",
                unless: Some("all"),
            }],
            summary: "Open a thing",
        },
        CommandSpec {
            name: "list",
            options: &[],
            positionals: &[],
            summary: "List the things",
        },
    ],
    name: "tool",
    options: &[Flag::choice("color", &["auto", "never"], "Color the text")],
    summary: "A tool",
    version: "1.2.3",
};

fn parse(words: &[&str]) -> Outcome {
    PROGRAM.parse(words.iter().map(OsString::from))
}

fn refusal(words: &[&str]) -> String {
    match parse(words) {
        Outcome::Refused(refusal, _) => refusal,
        _ => panic!("{words:?} was not refused"),
    }
}

fn run(words: &[&str]) -> Parsed {
    match parse(words) {
        Outcome::Run(parsed) => parsed,
        Outcome::Refused(refusal, _) => panic!("{words:?}: {refusal}"),
        _ => panic!("{words:?} did not run"),
    }
}

#[test]
fn a_command_takes_its_options_before_between_and_after_its_arguments() {
    let parsed = run(&[
        "--color", "never", "show", "--name", "a", "file", "--name=b", "-a",
        "--count", "7",
    ]);
    assert_eq!(parsed.command.unwrap().name, "show");
    assert_eq!(parsed.all("name"), ["a", "b"]);
    assert_eq!(parsed.one("name"), Some("b"));
    assert!(parsed.has("all"));
    assert_eq!(parsed.number("count"), Some(7));
    assert_eq!(parsed.one("color"), Some("never"));
    assert_eq!(parsed.positionals, ["file"]);
}

#[test]
fn a_default_is_given_when_the_option_is_not_and_only_then() {
    let parsed = run(&["show", "file"]);
    assert_eq!(parsed.number("count"), Some(3));
    assert!(!parsed.has("count"));
    assert_eq!(parsed.one("form"), None);
    assert!(parsed.all("name").is_empty());
}

#[test]
fn everything_after_two_dashes_is_an_argument() {
    let parsed = run(&["show", "--", "--all"]);
    assert_eq!(parsed.positionals, ["--all"]);
    assert!(!parsed.has("all"));
}

#[test]
fn help_and_version_are_asked_for_anywhere_they_make_sense() {
    assert!(matches!(parse(&[]), Outcome::Help(None)));
    assert!(matches!(parse(&["--help"]), Outcome::Help(None)));
    assert!(matches!(parse(&["-V"]), Outcome::Version));
    assert!(matches!(parse(&["--version"]), Outcome::Version));
    for words in [&["show", "-h"][..], &["show", "--bad", "--help"][..]] {
        match parse(words) {
            Outcome::Help(Some(command)) => assert_eq!(command.name, "show"),
            Outcome::Refused(refusal, _) => {
                assert_eq!(refusal, "unexpected argument '--bad' found");
            }
            _ => panic!("{words:?}"),
        }
    }
    assert_eq!(
        refusal(&["show", "--version"]),
        "unexpected argument '--version' found"
    );
}

#[test]
fn the_flags_of_a_command_are_listed_in_order_with_those_of_the_program() {
    let names: Vec<&str> = PROGRAM
        .flags(PROGRAM.command("show"))
        .iter()
        .map(|flag| flag.long)
        .collect();
    assert_eq!(names, ["all", "color", "count", "form", "name", "quiet"]);
    let flag = PROGRAM.command("show").unwrap().flag("form").unwrap();
    assert_eq!(flag.shown(), "--form short|long");
    assert_eq!(flag.shown_value().as_deref(), Some("short|long"));
    assert_eq!(PROGRAM.flags(None).len(), 1);
}

#[test]
fn what_cannot_be_understood_is_refused_with_the_reason() {
    for (words, expected) in [
        (&["frob"][..], "unrecognized command 'frob'"),
        (&["show", "--nope"], "unexpected argument '--nope' found"),
        (&["show", "-x"], "unexpected argument '-x' found"),
        (&["show", "-ab"], "unexpected argument '-ab' found"),
        (&["list", "extra"], "unexpected argument 'extra' found"),
        (&["show", "a", "b"], "unexpected argument 'b' found"),
        (
            &["show", "f", "--name"],
            "a value is required for '--name NAME' but none was supplied",
        ),
        (&["show", "f", "--all=yes"], "'--all' takes no value"),
        (
            &["show", "f", "--form", "wide"],
            "invalid value 'wide' for '--form short|long'; the values are \
             short, long",
        ),
        (
            &["show", "--all", "--count", "10"],
            "invalid value '10' for '--count N'; it is a whole number up to 9",
        ),
        (
            &["show", "--all", "--count", "-1"],
            "invalid value '-1' for '--count N'; it is a whole number up to 9",
        ),
        (
            &["show", "f", "--form", "long", "--form", "short"],
            "the argument '--form short|long' cannot be used multiple times",
        ),
        (
            &["show", "--all", "--all"],
            "the argument '--all' cannot be used multiple times",
        ),
        (
            &["show", "f", "--count", "2"],
            "'--count N' needs '--all' as well",
        ),
        (
            &["show", "--quiet", "--all"],
            "the argument '--quiet' cannot be used with '--all'",
        ),
        (
            &["open"],
            "the following required argument was not provided: FILE",
        ),
        (
            &["open", "--all", "f"],
            "the argument '--all' cannot be used with 'FILE'",
        ),
    ] {
        assert_eq!(refusal(words), expected, "{words:?}");
    }
}

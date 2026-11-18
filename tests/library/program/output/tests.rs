// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::program::output::*;

#[test]
fn a_status_line_right_aligns_its_verb_in_twelve_columns() {
    assert_eq!(
        plain(&status_line("Compiling", "hello.cc")),
        "   Compiling hello.cc"
    );
    assert_eq!(
        plain(&status_line("Finished", "release")),
        "    Finished release"
    );
}

#[test]
fn asked_for_color_paints_each_line_as_cargo_does() {
    assert_eq!(paint("error:", ERROR), "\u{1b}[1;31merror:\u{1b}[0m");
    assert_eq!(paint("warning:", WARNING), "\u{1b}[1;33mwarning:\u{1b}[0m");
    assert_eq!(
        paint("   Compiling", GOOD),
        "\u{1b}[1;32m   Compiling\u{1b}[0m"
    );
}

#[test]
fn clipped_names_say_that_they_were_cut() {
    assert_eq!(clip("short", 10), "short");
    assert_eq!(clip("a-very-long-symbol-name", 10), "a-very-...");
}

#[test]
fn errors_and_warnings_start_with_their_word_and_a_colon() {
    assert_eq!(
        plain(&error_line("no project here")),
        "error: no project here"
    );
    assert_eq!(
        plain(&warning_line("a.cc is newer")),
        "warning: a.cc is newer"
    );
}

#[test]
fn field_values_and_last_table_cells_continue_beneath_themselves() {
    let mut text = String::new();
    fields(&mut text, &[("Key", "value ".repeat(30))]);
    let text = plain(&text);
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.len() > 1);
    assert!(lines[1].starts_with("        value"), "{text}");
    assert!(lines.iter().all(|line| line.len() <= WIDTH), "{text}");

    let mut text = String::new();
    table(
        &mut text,
        &["NAME", "NOTE"],
        &[["a".into(), "note ".repeat(30)]],
    );
    let text = plain(&text);
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.len() > 2, "{text}");
    assert!(lines[2].starts_with("        note"), "{text}");
    assert!(lines.iter().all(|line| line.len() <= WIDTH), "{text}");
}

#[test]
fn long_text_wraps_at_spaces_and_no_line_passes_the_width() {
    let text = "word ".repeat(40);
    let wrapped = wrap(&text, 9, 2);
    let lines: Vec<&str> = wrapped.lines().collect();
    assert!(lines.len() > 1);
    assert!(lines[0].len() + 9 <= WIDTH);
    for line in &lines[1..] {
        assert!(line.starts_with("  word"), "{line:?}");
        assert!(line.len() <= WIDTH, "{line:?}");
    }
    let path = "x".repeat(WIDTH + 10);
    assert_eq!(wrap(&format!("see {path} now"), 0, 2).lines().count(), 3);
}

#[test]
fn messages_keep_blocks_that_are_already_laid_out() {
    let text = format!("{}\n\nChoices:\n  one  two", "word ".repeat(30));
    let message = message(&text);
    assert!(message.ends_with("\n\nChoices:\n  one  two"), "{message}");
    assert!(message.lines().all(|line| line.len() <= WIDTH), "{message}");
}

#[test]
fn names_fill_columns_downwards_within_the_width() {
    let names: Vec<String> = (0..23)
        .map(|index| format!("project-number-{index:02}"))
        .collect();
    let listed = columns(&names);
    assert!(listed.lines().all(|line| line.len() <= WIDTH), "{listed}");
    let first: Vec<&str> =
        listed.lines().next().unwrap().split_whitespace().collect();
    assert!(first.len() > 1, "{listed}");
    assert_eq!(first[0], "project-number-00");
    assert!(
        listed
            .lines()
            .nth(1)
            .unwrap()
            .trim_start()
            .starts_with("project-number-01")
    );
    assert_eq!(listed.split_whitespace().count(), names.len());
    assert!(
        choices("Projects", &names)
            .starts_with("\n\nProjects:\n  project-number-00")
    );
    assert_eq!(choices::<&str>("Projects", &[]), "\n\nProjects: none");
}

#[test]
fn plain_text_keeps_every_control_sequence_but_color() {
    assert_eq!(plain(&paint("warning:", WARNING)), "warning:");
    assert_eq!(
        plain("\u{1b}[2Kline\u{1b}[1;36m!\u{1b}[0m"),
        "\u{1b}[2Kline!"
    );
}

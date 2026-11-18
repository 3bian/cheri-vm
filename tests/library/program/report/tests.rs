// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::program::output::Format;
use crate::program::report::{deliver, document, format};
use crate::program::temporary::TemporaryDirectory;
use std::path::Path;

#[test]
fn a_document_starts_with_its_format_version() {
    let document = document([("status".to_owned(), "ok".into())]);
    assert_eq!(
        serde_json::to_string(&document).unwrap(),
        "{\"format_version\":1,\"status\":\"ok\"}"
    );
}

#[test]
fn a_report_sent_to_a_file_ends_with_one_line_end() {
    let folder = TemporaryDirectory::new().unwrap();
    let file = folder.path().join("report.txt");
    let shell =
        crate::program::shell::Shell::new(crate::program::output::Color::Never);
    deliver(&shell, "\u{1b}[1mok\u{1b}[0m", Some(&file)).unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "ok\n");
    deliver(&shell, "ok\n", Some(&file)).unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "ok\n");
}

#[test]
fn the_file_name_chooses_the_form_unless_one_is_named() {
    let allowed = crate::program::output::REPORT_FORMATS;
    let chosen = |name: &str| format(None, Some(Path::new(name)), allowed);
    assert!(matches!(chosen("report.json"), Format::Json));
    assert!(matches!(chosen("report.toml"), Format::Toml));
    assert!(matches!(chosen("report.txt"), Format::Text));
    assert!(matches!(format(None, None, allowed), Format::Text));
    assert!(matches!(
        format(Some(Format::Text), Some(Path::new("report.json")), allowed),
        Format::Text
    ));
    let plans = &[Format::Text, Format::Json, Format::CompilationDatabase];
    assert!(matches!(
        format(None, Some(Path::new("compile_commands.json")), plans),
        Format::CompilationDatabase
    ));
    assert!(matches!(
        format(None, Some(Path::new("compile_commands.json")), allowed),
        Format::Json
    ));
}

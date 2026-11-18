// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::shell::Shell;
use crate::program::output::{self, Format, OutputError, Report};
use std::path::Path;

pub fn deliver(
    shell: &Shell,
    text: &str,
    output: Option<&Path>,
) -> std::io::Result<()> {
    let line_end = if text.ends_with('\n') { "" } else { "\n" };
    if text.is_empty() && output.is_none() {
        return Ok(());
    }
    match output {
        Some(file) => {
            std::fs::write(file, format!("{}{line_end}", output::plain(text)))
        }
        None => {
            shell.stdout(&format!("{text}{line_end}"));
            Ok(())
        }
    }
}

pub fn document(
    fields: impl IntoIterator<Item = (String, serde_json::Value)>,
) -> serde_json::Map<String, serde_json::Value> {
    let mut document = serde_json::Map::new();
    document.insert("format_version".into(), output::FORMAT_VERSION.into());
    document.extend(fields);
    document
}

pub fn emit(
    shell: &Shell,
    report: &dyn Report,
    format: Format,
    output: Option<&Path>,
) -> Result<(), OutputError> {
    let text = output::render_text(report, format)?;
    deliver(shell, &text, output)
        .map_err(|error| OutputError::Message(error.to_string()))
}

pub fn format(
    chosen: Option<Format>,
    output: Option<&Path>,
    allowed: &[Format],
) -> Format {
    chosen
        .or_else(|| output.and_then(|file| Format::for_file(file, allowed)))
        .or_else(|| allowed.first().copied())
        .unwrap_or(Format::Text)
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod color;
mod format;
mod output_error;

pub use color::Color;
pub use format::Format;
pub use output_error::OutputError;

mod report;

pub mod toml;

pub use report::{Built, Report, render, render_text};

use serde::Serialize;

pub(crate) const ERROR: &str = "1;31";
pub const FORMAT_VERSION: u32 = 1;
pub(crate) const GOOD: &str = "1;32";

pub const REPORT_FORMATS: &[Format] =
    &[Format::Text, Format::Json, Format::Toml];

const STATUS_WIDTH: usize = 12;
pub(crate) const WARNING: &str = "1;33";
pub const WIDTH: usize = 80;

pub fn bold(text: &str) -> String {
    paint(text, "1")
}

pub fn choices<S: AsRef<str>>(title: &str, names: &[S]) -> String {
    if names.is_empty() {
        return format!("\n\n{title}: none");
    }
    format!("\n\n{title}:\n{}", columns(names).trim_end())
}

pub fn clip(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_owned();
    }
    let kept: String = text.chars().take(width.saturating_sub(3)).collect();
    format!("{kept}...")
}

pub fn columns<S: AsRef<str>>(names: &[S]) -> String {
    let widest = names
        .iter()
        .map(|name| name.as_ref().chars().count())
        .max()
        .unwrap_or(0);
    let per_row = ((WIDTH - 2 + 2) / (widest + 2)).clamp(1, names.len().max(1));
    let rows = names.len().div_ceil(per_row);
    let mut out = String::new();
    for row in 0..rows {
        let mut line = String::from(" ");
        for column in 0..per_row {
            let Some(name) = names.get(column * rows + row) else {
                continue;
            };
            line.push_str(&format!(" {:<widest$} ", name.as_ref()));
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

pub fn dim(text: &str) -> String {
    paint(text, "2")
}

pub fn document<T: Serialize>(value: &T) -> serde_json::Value {
    versioned(serde_json::to_value(value).unwrap_or_default())
}

pub fn error_line(text: &str) -> String {
    format!("{} {}", paint("error:", ERROR), message(text))
}

pub fn fields(text: &mut String, pairs: &[(&str, String)]) {
    let width = pairs.iter().map(|(key, _)| key.len()).max().unwrap_or(0) + 1;
    let indent = 2 + width + 2;
    for (key, value) in pairs {
        let label = format!("{key}:");
        let mut lines =
            wrap_words(value, WIDTH.saturating_sub(indent).max(20)).into_iter();
        text.push_str(&format!(
            "  {}  {}\n",
            dim(&format!("{label:<width$}")),
            status(&lines.next().unwrap_or_default())
        ));
        for line in lines {
            text.push_str(&format!("{}{line}\n", " ".repeat(indent)));
        }
    }
}

pub fn heading(text: &mut String, title: &str) {
    if !text.is_empty() {
        text.push('\n');
    }
    text.push_str(&paint(title, GOOD));
    text.push('\n');
}

pub fn line(text: &str) -> String {
    let text = shorten_paths(text);
    let indent = text.len() - text.trim_start().len();
    format!("{}{}", " ".repeat(indent), wrap(&text, indent, indent + 2))
}

pub fn lines<I: IntoIterator<Item = S>, S: AsRef<str>>(
    text: &mut String,
    items: I,
) {
    for item in items {
        for (index, line) in item.as_ref().lines().enumerate() {
            if line.is_empty() {
                text.push('\n');
            } else if index == 0 {
                text.push_str(&format!("  {}\n", wrap(line, 2, 4)));
            } else if line.starts_with(' ') {
                text.push_str(&format!("    {line}\n"));
            } else {
                text.push_str(&format!("    {}\n", wrap(line, 4, 4)));
            }
        }
    }
}

pub(crate) fn message(text: &str) -> String {
    const PREFIX: usize = "error: ".len();
    let text = shorten_paths(text);
    let mut out = String::new();
    for (index, line) in text.lines().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        if line.starts_with(' ') {
            out.push_str(line);
        } else if index == 0 {
            out.push_str(&wrap(line, PREFIX, 2));
        } else {
            out.push_str(&wrap(line, 0, 2));
        }
    }
    out
}

pub(crate) fn paint(text: &str, code: &str) -> String {
    if text.is_empty() {
        String::new()
    } else {
        format!("\u{1b}[{code}m{text}\u{1b}[0m")
    }
}

pub fn path(path: &std::path::Path) -> String {
    let inside = |directory: Option<std::path::PathBuf>| {
        let directory =
            directory.filter(|directory| directory.parent().is_some())?;
        path.strip_prefix(directory).ok().map(|p| p.to_path_buf())
    };
    if let Some(relative) = inside(std::env::current_dir().ok()) {
        return if relative.as_os_str().is_empty() {
            ".".into()
        } else {
            relative.display().to_string()
        };
    }
    let home =
        std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" });
    match inside(home.map(std::path::PathBuf::from)) {
        Some(relative) if !relative.as_os_str().is_empty() => {
            format!("~{}{}", std::path::MAIN_SEPARATOR, relative.display())
        }
        _ => path.display().to_string(),
    }
}

pub fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("\u{1b}[") {
        let (before, sequence) = rest.split_at(start);
        out.push_str(before);
        let parameters = sequence
            .char_indices()
            .skip(2)
            .find(|(_, code)| !(code.is_ascii_digit() || *code == ';'));
        match parameters {
            Some((end, 'm')) => {
                rest = sequence.get(end + 1..).unwrap_or_default()
            }
            _ => {
                out.push('\u{1b}');
                rest = sequence.get(1..).unwrap_or_default();
            }
        }
    }
    out.push_str(rest);
    out
}

pub fn shorten_paths(text: &str) -> String {
    match std::env::current_dir() {
        Ok(directory) if directory.parent().is_some() => {
            let prefix =
                format!("{}{}", directory.display(), std::path::MAIN_SEPARATOR);
            text.replace(&prefix, "")
        }
        _ => text.to_owned(),
    }
}

pub(crate) fn status(word: &str) -> String {
    match status_code(word) {
        Some(code) => paint(word, code),
        None => word.to_owned(),
    }
}

fn status_code(word: &str) -> Option<&'static str> {
    match word {
        "ok" | "valid" | "supported" | "passed" => Some("32"),
        "missing" | "invalid" | "not supported" | "failed" | "error" => {
            Some("31")
        }
        "skipped" | "unknown" | "third-party" | "unresolved" => Some("33"),
        _ => None,
    }
}

pub fn status_line(verb: &str, text: &str) -> String {
    format!(
        "{} {}",
        paint(&format!("{verb:>STATUS_WIDTH$}"), GOOD),
        shorten_paths(text)
    )
}

pub fn table<const N: usize>(
    text: &mut String,
    headers: &[&str; N],
    rows: &[[String; N]],
) {
    let mut widths = headers.map(str::len);
    for row in rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }
    let last_start =
        2 + widths[..N - 1].iter().map(|width| width + 2).sum::<usize>();
    let last_room = WIDTH.saturating_sub(last_start).max(16);
    let mut line = |cells: [&str; N], header: bool| {
        let mut rendered = String::from(" ");
        let mut continuation = Vec::new();
        for (index, (cell, width)) in cells.iter().zip(widths).enumerate() {
            if index + 1 == N {
                let mut parts = wrap_words(cell, last_room).into_iter();
                let first = parts.next().unwrap_or_default();
                let first = if header { dim(&first) } else { status(&first) };
                rendered.push_str(&format!(" {first}"));
                continuation.extend(parts);
            } else {
                let padding = width - cell.chars().count();
                let cell = if header { dim(cell) } else { status(cell) };
                rendered.push_str(&format!(" {cell}{} ", " ".repeat(padding)));
            }
        }
        text.push_str(rendered.trim_end());
        text.push('\n');
        for part in continuation {
            text.push_str(&format!("{}{part}\n", " ".repeat(last_start)));
        }
    };
    line(*headers, true);
    for row in rows {
        line(std::array::from_fn(|index| row[index].as_str()), false);
    }
}

pub fn versioned(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(fields)
            if !fields.contains_key("format_version") =>
        {
            let mut document = serde_json::Map::new();
            document.insert("format_version".into(), FORMAT_VERSION.into());
            document.extend(fields);
            document.into()
        }
        other => other,
    }
}

pub fn warning_line(text: &str) -> String {
    format!("{} {}", paint("warning:", WARNING), message(text))
}

pub(crate) fn wrap(text: &str, first: usize, rest: usize) -> String {
    let mut out = String::new();
    let mut column = first;
    let mut start_of_line = true;
    for word in text.split_whitespace() {
        let length = word.chars().count();
        if !start_of_line && column + 1 + length > WIDTH {
            out.push('\n');
            out.push_str(&" ".repeat(rest));
            column = rest;
            start_of_line = true;
        }
        if !start_of_line {
            out.push(' ');
            column += 1;
        }
        out.push_str(word);
        column += length;
        start_of_line = false;
    }
    out
}

fn wrap_words(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty()
            && line.chars().count() + 1 + word.chars().count() > width
        {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}

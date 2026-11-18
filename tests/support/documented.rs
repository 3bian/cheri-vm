// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::programs::repository;
use std::path::{Path, PathBuf};

pub const PROGRAM: &str = "cheri-vm";

pub struct DocumentedCommand {
    pub file: PathBuf,
    pub line: usize,
    pub words: Vec<String>,
}

impl DocumentedCommand {
    pub fn place(&self) -> String {
        let root = repository();
        let file = self.file.strip_prefix(root).unwrap_or(&self.file).display();
        format!("{file}:{}", self.line)
    }
}

pub fn documented_commands() -> Vec<DocumentedCommand> {
    let mut commands = Vec::new();
    for file in documents() {
        let text = std::fs::read_to_string(&file).unwrap();
        let mut block = None;
        let mut pending: Option<(usize, String)> = None;
        for (index, line) in text.lines().enumerate() {
            if let Some(language) = line.trim_start().strip_prefix("```") {
                block = match block {
                    None => Some(language.trim() == "console"),
                    Some(_) => None,
                };
                pending = None;
                continue;
            }
            if block != Some(false) {
                continue;
            }
            let (start, joined) = match pending.take() {
                Some((start, begun)) => (start, begun + " " + line.trim()),
                None => (index + 1, line.trim_start().to_owned()),
            };
            if let Some(more) = joined.strip_suffix('\\') {
                pending = Some((start, more.trim_end().to_owned()));
                continue;
            }
            let words = shell_words(&joined);
            if words.first().is_some_and(|first| first == PROGRAM) {
                commands.push(DocumentedCommand {
                    file: file.clone(),
                    line: start,
                    words,
                });
            }
        }
    }
    commands
}

fn documents() -> Vec<PathBuf> {
    let mut files = Vec::new();
    markdown(&repository(), &mut files);
    markdown(&repository().join("../book"), &mut files);
    files.sort();
    files
}

fn markdown(folder: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name != "target") {
                markdown(&path, found);
            }
        } else if path.extension().is_some_and(|ending| ending == "md") {
            found.push(path);
        }
    }
}

fn shell_words(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut quote = None;
    for character in line.chars() {
        match (quote, character) {
            (Some(open), _) if character == open => quote = None,
            (Some(_), _) => word.push(character),
            (None, '\'' | '"') => {
                quote = Some(character);
                started = true;
            }
            (None, '#' | '<' | '>' | '|' | ';' | '&') if !started => break,
            (None, ' ' | '\t') => {
                if started {
                    words.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            (None, _) => {
                word.push(character);
                started = true;
            }
        }
    }
    if started {
        words.push(word);
    }
    words
}

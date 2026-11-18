// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::diagnostic::{Diagnostic, Result};
use super::output::Color;
use std::path::PathBuf;

pub const FILE: &str = "config.toml";
pub const FOLDER: &str = "cheri-vm";
pub const HOME_VARIABLE: &str = "CHERI_VM_HOME";
pub const PROFILES_FOLDER: &str = "profiles";

pub fn color() -> Color {
    let chosen = file()
        .and_then(|file| std::fs::read_to_string(file).ok())
        .and_then(|text| text.parse::<toml::Table>().ok())
        .and_then(|table| table.get("color")?.as_str().map(str::to_owned));
    match chosen.as_deref() {
        Some("always") => Color::Always,
        Some("never") => Color::Never,
        _ => Color::Auto,
    }
}

pub fn file() -> Option<PathBuf> {
    Some(home()?.join(FILE))
}

pub fn home() -> Option<PathBuf> {
    let set =
        |name: &str| std::env::var_os(name).filter(|value| !value.is_empty());
    if let Some(folder) = set(HOME_VARIABLE) {
        return Some(PathBuf::from(folder));
    }
    let directory = if cfg!(windows) {
        set("APPDATA").map(PathBuf::from)
    } else {
        set("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|directory| directory.is_absolute())
            .or_else(|| {
                set("HOME").map(|home| PathBuf::from(home).join(".config"))
            })
    };
    Some(directory?.join(FOLDER))
}

pub fn profiles() -> Option<PathBuf> {
    Some(home()?.join(PROFILES_FOLDER))
}

pub fn read_environment(name: &str) -> Result<Option<String>> {
    match std::env::var(name) {
        Ok(value) if value.is_empty() => {
            Err(Diagnostic::new(format!("{name} must not be empty")))
        }
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => {
            Err(Diagnostic::new(format!("{name} must be valid UTF-8")))
        }
    }
}

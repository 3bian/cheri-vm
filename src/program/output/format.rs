// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Text,
    Json,
    Toml,
    Shell,
    CompilationDatabase,
    Bytes,
    Cyclonedx,
    Spdx,
    Junit,
}

impl Format {
    pub fn for_file(file: &std::path::Path, allowed: &[Self]) -> Option<Self> {
        let name = file.file_name()?.to_str()?;
        let chosen = if name == "compile_commands.json" {
            Self::CompilationDatabase
        } else if name.ends_with(".spdx.json") {
            Self::Spdx
        } else if name.ends_with(".cdx.json") {
            Self::Cyclonedx
        } else if name.ends_with(".xml") {
            Self::Junit
        } else if name.ends_with(".json") {
            Self::Json
        } else if name.ends_with(".toml") {
            Self::Toml
        } else if name.ends_with(".sh") {
            Self::Shell
        } else {
            return None;
        };
        if allowed.contains(&chosen) {
            Some(chosen)
        } else if name.ends_with(".json") && allowed.contains(&Self::Json) {
            Some(Self::Json)
        } else {
            None
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Json => "json",
            Self::Toml => "toml",
            Self::Shell => "shell",
            Self::CompilationDatabase => "compilation-database",
            Self::Bytes => "bytes",
            Self::Cyclonedx => "cyclonedx",
            Self::Spdx => "spdx",
            Self::Junit => "junit",
        }
    }

    pub fn named(name: &str) -> Option<Self> {
        match name {
            "text" => Some(Self::Text),
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            _ => None,
        }
    }
}

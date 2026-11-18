// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use cheri_vm::program::diagnostic::Diagnostic;
use cheri_vm::program::output::OutputError;

#[derive(Debug)]
pub(crate) enum AppError {
    Config(Diagnostic),
    Io(std::io::Error),
    Json(serde_json::Error),
    Message(String),
    Output(OutputError),
}

impl From<Diagnostic> for AppError {
    fn from(error: Diagnostic) -> Self {
        Self::Config(error)
    }
}

impl From<OutputError> for AppError {
    fn from(error: OutputError) -> Self {
        Self::Output(error)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl std::error::Error for AppError {}

impl std::fmt::Display for AppError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(error) => error.fmt(formatter),
            Self::Io(error) => error.fmt(formatter),
            Self::Json(error) => error.fmt(formatter),
            Self::Message(message) => formatter.write_str(message),
            Self::Output(error) => error.fmt(formatter),
        }
    }
}

pub(crate) type Result<T> = std::result::Result<T, AppError>;

pub(crate) fn require_utf8_path(path: &std::path::Path) -> Result<()> {
    path.to_str().map(|_| ()).ok_or_else(|| {
        AppError::Message(format!("path is not UTF-8: {}", path.display()))
    })
}

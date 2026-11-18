// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::{fmt, path::Path};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
}

impl Diagnostic {
    pub fn io(path: &Path, error: impl fmt::Display) -> Self {
        Self::new(format!("{}: {error}", path.display()))
    }

    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl From<&str> for Diagnostic {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

impl From<crate::program::output::OutputError> for Diagnostic {
    fn from(error: crate::program::output::OutputError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<serde_json::Error> for Diagnostic {
    fn from(error: serde_json::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<std::io::Error> for Diagnostic {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<String> for Diagnostic {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl std::error::Error for Diagnostic {}

pub type Result<T> = std::result::Result<T, Diagnostic>;

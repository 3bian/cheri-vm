// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Format;

#[derive(Debug)]
pub enum OutputError {
    Json(serde_json::Error),
    Message(String),
    Unsupported(Format),
}

impl From<serde_json::Error> for OutputError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl std::error::Error for OutputError {}

impl std::fmt::Display for OutputError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(error) => error.fmt(formatter),
            Self::Message(message) => formatter.write_str(message),
            Self::Unsupported(format) => write!(
                formatter,
                "this report cannot be written as {}",
                format.name()
            ),
        }
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod built;

pub use built::Built;

use super::{Format, OutputError};

pub trait Report {
    fn code(&self) -> u8 {
        0
    }

    fn document(&self) -> serde_json::Value;

    fn other(&self, format: Format) -> Option<Result<Vec<u8>, OutputError>> {
        let _ = format;
        None
    }

    fn text(&self) -> String;
}

pub fn render(
    report: &dyn Report,
    format: Format,
) -> Result<Vec<u8>, OutputError> {
    match format {
        Format::Text => Ok(report.text().into_bytes()),
        Format::Json => {
            Ok(serde_json::to_string_pretty(&report.document())?.into_bytes())
        }
        Format::Toml => Ok(super::toml::write(&report.document()).into_bytes()),
        other => report
            .other(other)
            .unwrap_or(Err(OutputError::Unsupported(other))),
    }
}

pub fn render_text(
    report: &dyn Report,
    format: Format,
) -> Result<String, OutputError> {
    String::from_utf8(render(report, format)?)
        .map_err(|_| OutputError::Message("the report is not text".into()))
}

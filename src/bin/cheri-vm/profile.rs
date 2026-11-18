// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::cli::ProfileArgs;
use crate::error::{AppError, Result};
use cheri_vm::profile::built_in;
use cheri_vm::program::command::{Context, Run};
use cheri_vm::program::output::{self, Built, Format, REPORT_FORMATS, Report};
use std::path::Path;

impl Run for ProfileArgs {
    type Error = AppError;
    const FORMATS: &'static [Format] = REPORT_FORMATS;

    fn format(&self) -> Option<Format> {
        self.format
    }

    fn output(&self) -> Option<&Path> {
        self.output.as_deref()
    }

    fn run(self, _context: &Context) -> Result<Box<dyn Report>> {
        match self.name.filter(|_| !self.list) {
            Some(name) => shown(&name),
            None => listed(),
        }
    }
}

fn listed() -> Result<Box<dyn Report>> {
    let rows = built_in::names()
        .into_iter()
        .map(|name| {
            let table =
                parsed(&name, built_in::text(&name).unwrap_or_default())?;
            let description = table
                .get("description")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .to_owned();
            Ok([name, description])
        })
        .collect::<Result<Vec<[String; 2]>>>()?;
    let mut text = String::new();
    output::heading(&mut text, "Profiles");
    output::table(&mut text, &["NAME", "DESCRIPTION"], &rows);
    let profiles: Vec<serde_json::Value> = rows
        .iter()
        .map(|[name, description]| {
            serde_json::json!({
                "name": name,
                "description": description,
            })
        })
        .collect();
    Ok(Box::new(Built {
        text,
        document: output::versioned(
            serde_json::json!({ "profiles": profiles }),
        ),
        code: 0,
    }))
}

fn parsed(name: &str, text: &str) -> Result<toml::Table> {
    toml::from_str(text)
        .map_err(|error| AppError::Message(format!("{name}: {error}")))
}

fn shown(name: &str) -> Result<Box<dyn Report>> {
    let Some(text) = built_in::text(name) else {
        return Err(AppError::Message(format!(
            "no built-in profile is named {name:?}{}",
            output::choices("Profiles", &built_in::names())
        )));
    };
    let document = serde_json::to_value(parsed(name, text)?)?;
    Ok(Box::new(Built {
        text: text.to_owned(),
        document,
        code: 0,
    }))
}

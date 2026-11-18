// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use serde_json::{Map, Value};
use std::fmt::Write;

fn inline(value: &Value) -> String {
    match value {
        Value::Null => "\"\"".to_owned(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(value) => number(value),
        Value::String(text) => string(text),
        Value::Array(items) => {
            let items: Vec<String> = items
                .iter()
                .filter(|item| !item.is_null())
                .map(inline)
                .collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(fields) => {
            let fields: Vec<String> = fields
                .iter()
                .filter(|(_, value)| !value.is_null())
                .map(|(name, value)| {
                    format!("{} = {}", key(name), inline(value))
                })
                .collect();
            if fields.is_empty() {
                "{}".to_owned()
            } else {
                format!("{{ {} }}", fields.join(", "))
            }
        }
    }
}

fn key(name: &str) -> String {
    let bare = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'));
    if bare { name.to_owned() } else { string(name) }
}

fn number(number: &serde_json::Number) -> String {
    match number.as_f64() {
        Some(float)
            if number.is_f64() && float.fract() == 0.0 && float.is_finite() =>
        {
            format!("{float:.1}")
        }
        _ => number.to_string(),
    }
}

fn string(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');
    for character in text.chars() {
        match character {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(quoted, "\\u{:04X}", c as u32);
            }
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

fn table(text: &mut String, path: &str, fields: &Map<String, Value>) {
    for (name, value) in fields {
        if !value.is_null() && !value.is_object() && tables(value).is_none() {
            let _ = writeln!(text, "{} = {}", key(name), inline(value));
        }
    }
    for (name, value) in fields {
        let path = if path.is_empty() {
            key(name)
        } else {
            format!("{path}.{}", key(name))
        };
        if let Some(fields) = value.as_object() {
            let _ = write!(text, "\n[{path}]\n");
            table(text, &path, fields);
        } else if let Some(items) = tables(value) {
            for fields in items {
                let _ = write!(text, "\n[[{path}]]\n");
                table(text, &path, fields);
            }
        }
    }
}

fn tables(value: &Value) -> Option<Vec<&Map<String, Value>>> {
    let items = value.as_array().filter(|items| !items.is_empty())?;
    items.iter().map(Value::as_object).collect()
}

pub fn write(value: &Value) -> String {
    let mut text = String::new();
    match value {
        Value::Object(fields) => table(&mut text, "", fields),
        other => {
            let mut fields = Map::new();
            fields.insert("value".to_owned(), other.clone());
            table(&mut text, "", &fields);
        }
    }
    text
}

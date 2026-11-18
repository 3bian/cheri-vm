// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::program::output::toml::write;
use serde_json::{Value, json};

#[test]
fn a_document_reads_back_to_the_same_value() {
    let document = json!({
        "format_version": 1,
        "name": "hello \"world\"\n",
        "ratio": 0.5,
        "whole": 2.0,
        "negative": -7,
        "valid": true,
        "empty": [],
        "words": ["a", "b"],
        "nested": [[1, 2], [3]],
        "mixed": [{"a": 1}, 2],
        "odd key": {"a.b": {"deep": [1]}, "none": {}},
        "rows": [{
            "name": "one",
            "parts": [{"n": 1},
            {"n": 2}]},
            {"name": "two",
        }],
    });
    assert_eq!(read_back(&document), document);
}

#[test]
fn a_null_key_is_left_out() {
    let document = json!({
        "a": null,
        "b": {"c": null, "d": 1},
        "e": [{"f": null, "g": 2}],
    });
    assert_eq!(read_back(&document), without_nulls(&document));
    assert!(!write(&document).contains("a ="));
}

#[test]
fn a_value_that_is_not_a_table_is_written_under_the_key_value() {
    assert_eq!(write(&json!("text")), "value = \"text\"\n");
    assert_eq!(write(&json!(["a", "b"])), "value = [\"a\", \"b\"]\n");
}

fn read_back(document: &Value) -> Value {
    let text = write(document);
    let parsed: toml::Value = toml::from_str(&text).unwrap_or_else(|error| {
        panic!("{error}\n{text}");
    });
    serde_json::to_value(parsed).unwrap()
}

#[test]
fn top_level_values_come_before_the_first_table() {
    let text = write(&json!({"table": {"a": 1}, "last": 2}));
    assert_eq!(text, "last = 2\n\n[table]\na = 1\n");
}

fn without_nulls(value: &Value) -> Value {
    match value {
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .filter(|(_, value)| !value.is_null())
                .map(|(name, value)| (name.clone(), without_nulls(value)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .filter(|item| !item.is_null())
                .map(without_nulls)
                .collect(),
        ),
        other => other.clone(),
    }
}

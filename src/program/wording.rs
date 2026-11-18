// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

const PHRASES: [(&str, &str); 13] = [
    ("expected u8", "expected a whole number from 0 to 255"),
    ("expected u16", "expected a whole number from 0 to 65535"),
    (
        "expected u32",
        "expected a whole number from 0 to 4294967295",
    ),
    ("expected u64", "expected a whole number, zero or more"),
    ("expected usize", "expected a whole number, zero or more"),
    ("expected i32", "expected a whole number"),
    ("expected i64", "expected a whole number"),
    ("expected f32", "expected a number"),
    ("expected f64", "expected a number"),
    ("expected a sequence", "expected a list"),
    ("expected a map", "expected a table"),
    ("invalid type: sequence", "invalid type: list"),
    ("invalid type: map", "invalid type: table"),
];

pub fn plain(message: &str) -> String {
    let mut text = message.trim_end().replace('\n', " ");
    for (from, to) in PHRASES {
        text = replace_phrase(&text, from, to);
    }
    while let Some(start) = text.find("expected struct ") {
        let name_start = start + "expected struct ".len();
        let name_length = text[name_start..]
            .find(|character: char| {
                !character.is_alphanumeric() && character != '_'
            })
            .unwrap_or(text.len() - name_start);
        text.replace_range(start..name_start + name_length, "expected a table");
    }
    text
}

fn replace_phrase(text: &str, from: &str, to: &str) -> String {
    let mut result = String::new();
    let mut rest = text;
    while let Some(start) = rest.find(from) {
        let after = &rest[start + from.len()..];
        result.push_str(&rest[..start]);
        let whole =
            !after.starts_with(|character: char| character.is_alphanumeric());
        result.push_str(if whole { to } else { from });
        rest = after;
    }
    result.push_str(rest);
    result
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::program::wording::plain;

#[test]
fn other_messages_are_left_alone() {
    assert_eq!(
        plain("unknown variant `fast`, expected `debug` or `release`"),
        "unknown variant `fast`, expected `debug` or `release`"
    );
    assert_eq!(plain("expected u32x"), "expected u32x");
    assert_eq!(plain("missing field `name`"), "missing field `name`");
}

#[test]
fn type_names_of_the_program_become_plain_words() {
    assert_eq!(
        plain("invalid type: string \"abc\", expected u32"),
        "invalid type: string \"abc\", expected a whole number from 0 to \
         4294967295"
    );
    assert_eq!(
        plain("invalid type: integer `1`, expected struct RawThread"),
        "invalid type: integer `1`, expected a table"
    );
    assert_eq!(
        plain("invalid type: map, expected a sequence"),
        "invalid type: table, expected a list"
    );
    assert_eq!(
        plain(
            "invalid type: integer `5`, expected struct MemoryEntry\nin \
             `memory.ram0`\n"
        ),
        "invalid type: integer `5`, expected a table in `memory.ram0`"
    );
}

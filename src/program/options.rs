// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::arguments::Flag;

pub const COLOR: Flag = Flag::choice(
    "color",
    &["auto", "always", "never"],
    "Color text output: auto (only on a terminal, and not when NO_COLOR is \
     set), always or never. Defaults to the color setting, else auto",
);

pub const FORMAT: Flag = Flag::choice(
    "format",
    &["text", "json", "toml"],
    "Form of the report. Defaults to the form the name given to --output asks \
     for, else text",
);

pub const OUTPUT: Flag = Flag::value(
    "output",
    "FILE",
    "File to write the report to in place of standard output",
)
.shortened('o');

pub const PROFILE: Flag = Flag::value(
    "profile",
    "NAME|FILE",
    "Machine to run on: a profile built into the emulator, NAME.toml in the \
     profiles folder of your settings, or a profile file. Defaults to the \
     CHERI_VM_PROFILE environment variable",
);

pub const QUIET: Flag =
    Flag::switch("quiet", "Print errors only").shortened('q');

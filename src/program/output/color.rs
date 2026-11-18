// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Auto,
    Always,
    Never,
}

impl Color {
    pub fn colors(self, terminal: bool) -> bool {
        let allowed = std::env::var_os("NO_COLOR")
            .is_none_or(|value| value.is_empty())
            && std::env::var_os("TERM").is_none_or(|term| term != "dumb");
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => allowed && terminal,
        }
    }

    pub fn named(name: &str) -> Option<Self> {
        match name {
            "auto" => Some(Self::Auto),
            "always" => Some(Self::Always),
            "never" => Some(Self::Never),
            _ => None,
        }
    }
}

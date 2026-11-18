// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::program::output::{self, Color};
use std::io::{IsTerminal, Write};

pub const CLOSED_STREAM: u8 = 141;

#[derive(Clone, Copy, Debug)]
pub struct Shell {
    choice: Color,
    stderr: bool,
    stdout: bool,
}

impl Shell {
    pub fn choice(&self) -> Color {
        self.choice
    }

    pub fn error(&self, text: &str) {
        self.stderr(&format!("{}\n", output::error_line(text)));
    }

    pub fn for_stdout<'a>(&self, text: &'a str) -> std::borrow::Cow<'a, str> {
        shown(text, self.stdout)
    }

    pub fn new(choice: Color) -> Self {
        Self {
            choice,
            stdout: choice.colors(std::io::stdout().is_terminal()),
            stderr: choice.colors(std::io::stderr().is_terminal()),
        }
    }

    pub fn status(&self, verb: &str, object: &str) {
        self.stderr(&format!("{}\n", output::status_line(verb, object)));
    }

    pub fn stderr(&self, text: &str) {
        let text = shown(text, self.stderr);
        written(std::io::stderr().lock().write_all(text.as_bytes()));
    }

    pub fn stdout(&self, text: &str) {
        let text = self.for_stdout(text);
        let mut stream = std::io::stdout().lock();
        written(
            stream
                .write_all(text.as_bytes())
                .and_then(|()| stream.flush()),
        );
    }

    pub fn warning(&self, text: &str) {
        self.stderr(&format!("{}\n", output::warning_line(text)));
    }
}

fn shown(text: &str, colored: bool) -> std::borrow::Cow<'_, str> {
    if colored {
        text.into()
    } else {
        output::plain(text).into()
    }
}

fn written(result: std::io::Result<()>) {
    if result.is_err_and(|error| error.kind() == std::io::ErrorKind::BrokenPipe)
    {
        std::process::exit(i32::from(CLOSED_STREAM));
    }
}

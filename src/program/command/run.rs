// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Context;
use crate::program::diagnostic::Diagnostic;
use crate::program::output::{Format, OutputError, Report};
use std::path::Path;

pub trait Run: Sized {
    type Error: From<OutputError> + From<Diagnostic>;
    const FORMATS: &'static [Format];

    fn format(&self) -> Option<Format> {
        None
    }

    fn output(&self) -> Option<&Path> {
        None
    }

    fn run(self, context: &Context) -> Result<Box<dyn Report>, Self::Error>;
}

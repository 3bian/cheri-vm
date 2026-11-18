// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{CodeReport, FunctionReport, StackReport, text};
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct Report {
    pub instructions: u64,
    pub outside_code: u64,
    pub compartments: Vec<CodeReport>,
    pub functions: Vec<FunctionReport>,
    pub stacks: Vec<StackReport>,
}

impl crate::program::output::Report for Report {
    fn document(&self) -> serde_json::Value {
        crate::program::output::document(self)
    }

    fn text(&self) -> String {
        text(self)
    }
}

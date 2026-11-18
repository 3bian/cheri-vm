// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::Report;

#[derive(Default)]
pub struct Built {
    pub code: u8,
    pub document: serde_json::Value,
    pub text: String,
}

impl Report for Built {
    fn code(&self) -> u8 {
        self.code
    }

    fn document(&self) -> serde_json::Value {
        self.document.clone()
    }

    fn text(&self) -> String {
        self.text.clone()
    }
}

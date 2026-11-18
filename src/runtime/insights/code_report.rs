// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct CodeReport {
    pub name: String,
    pub instructions: u64,
    pub percent: f64,
    pub covered_bytes: u64,
    pub size: u64,
    pub coverage_percent: f64,
}

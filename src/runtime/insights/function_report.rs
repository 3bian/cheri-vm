// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct FunctionReport {
    pub name: String,
    pub instructions: u64,
    pub percent: f64,
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct StackReport {
    pub name: String,
    pub size: u64,
    pub peak_bytes: u64,
    pub peak_percent: f64,
}

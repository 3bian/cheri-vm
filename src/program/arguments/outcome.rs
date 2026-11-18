// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::{CommandSpec, Parsed};

pub enum Outcome {
    Help(Option<&'static CommandSpec>),
    Refused(String, Option<&'static CommandSpec>),
    Run(Parsed),
    Version,
}

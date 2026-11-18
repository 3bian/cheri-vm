// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use serde::Deserialize;

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Timing {
    #[default]
    Elapsed,
    PerStep,
}

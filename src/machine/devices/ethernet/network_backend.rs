// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::FrameTransport;

pub(crate) enum NetworkBackend {
    Host(Box<dyn FrameTransport>),
    InMemory,
}

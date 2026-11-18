// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::SocketOptions;

pub(crate) enum Host<T> {
    Linux(T),
    WebSocket(SocketOptions),
}

impl<T> Host<T> {
    pub(crate) fn socket(&self) -> Option<&SocketOptions> {
        match self {
            Self::Linux(_) => None,
            Self::WebSocket(socket) => Some(socket),
        }
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod host;
mod socket_options;

pub(crate) use host::Host;
pub(crate) use socket_options::SocketOptions;

use crate::machine::EmulatorError;

pub(crate) fn parts(url: &str) -> Result<(String, String), EmulatorError> {
    crate::web::handshake::parts(url).ok_or_else(|| {
        EmulatorError::Configuration(format!(
            "a WebSocket is ws, which serves the device at /ws/NAME, or \
             ws://HOST:PORT/PATH, which connects to a program that is \
             listening; not {url}; wss is not supported"
        ))
    })
}

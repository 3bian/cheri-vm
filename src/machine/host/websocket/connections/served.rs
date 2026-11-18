// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

pub(crate) struct Served {
    pub name: String,
    pub stream: std::net::TcpStream,
    pub url: String,
}

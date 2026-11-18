// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::net::TcpStream;

pub(super) enum Slot {
    Free(TcpStream),
    Gone,
    Taken,
}

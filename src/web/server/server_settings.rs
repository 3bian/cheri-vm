// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::net::IpAddr;
use std::path::PathBuf;

pub struct ServerSettings {
    pub devices: Vec<String>,
    pub host: IpAddr,
    pub port: u16,
    pub root: Option<PathBuf>,
}

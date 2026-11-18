// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct WebSettings {
    pub host: std::net::IpAddr,
    pub port: u16,
    pub root: Option<PathBuf>,
}

impl Default for WebSettings {
    fn default() -> Self {
        Self {
            host: std::net::Ipv4Addr::LOCALHOST.into(),
            port: 8080,
            root: None,
        }
    }
}

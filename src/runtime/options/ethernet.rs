// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::EmulatorError;

use super::socket::{Host, SocketOptions};
use crate::runtime::device_settings;

pub(crate) struct EthernetOptions {
    pub device: Option<String>,
    pub host: Host<String>,
}

impl EthernetOptions {
    pub(crate) fn parse(value: &str) -> Result<Self, EmulatorError> {
        let settings = device_settings::parse(
            value,
            "--ethernet",
            &["device", "interface", "timeout", "ws"],
        )?;
        let host = match SocketOptions::parse(
            &settings,
            "--ethernet",
            &["interface"],
            &[],
        )? {
            Some(socket) => Host::WebSocket(socket),
            None => Host::Linux(
                (*settings.get("interface").ok_or_else(|| {
                    EmulatorError::Configuration(
                        "--ethernet requires interface=NAME, ws or ws://HOST:PORT/PATH".into(),
                    )
                })?)
                .to_owned(),
            ),
        };
        Ok(Self {
            device: settings.get("device").map(|value| (*value).to_owned()),
            host,
        })
    }
}

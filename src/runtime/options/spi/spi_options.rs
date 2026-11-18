// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::parse_unsigned;
use super::super::socket::{Host, SocketOptions};
use super::SpiBus;
use crate::machine::EmulatorError;
use crate::runtime::device_settings;

pub(crate) struct SpiOptions {
    pub device: Option<String>,
    pub host: Host<SpiBus>,
    pub select: u32,
    pub write_only: bool,
}

impl SpiOptions {
    pub(crate) fn parse(value: &str) -> Result<Self, EmulatorError> {
        let settings = device_settings::parse(
            value,
            "--spi",
            &[
                "device",
                "path",
                "select",
                "speed",
                "timeout",
                "ws",
                "write-only",
            ],
        )?;
        let select = settings
            .get("select")
            .map_or(Ok(0), |value| parse_unsigned(value, "SPI select"))?;
        let speed = settings
            .get("speed")
            .map_or(Ok(500_000), |value| parse_unsigned(value, "SPI speed"))?;
        if select >= 4 || !(1..=50_000_000).contains(&speed) {
            return Err(EmulatorError::Configuration(
                "--spi requires select=0..3 and speed=1..50000000 Hz".into(),
            ));
        }
        let write_only = settings
            .get("write-only")
            .map(|value| value.parse::<bool>())
            .transpose()
            .map_err(|_| {
                EmulatorError::Configuration(
                    "SPI write-only requires true or false".into(),
                )
            })?
            .unwrap_or(false);
        let host = match SocketOptions::parse(
            &settings,
            "--spi",
            &["path"],
            &["speed"],
        )? {
            Some(socket) => Host::WebSocket(socket),
            None => Host::Linux(SpiBus {
                path: (*settings.get("path").ok_or_else(|| {
                    EmulatorError::Configuration(
                        "--spi requires path=PATH, ws or ws://HOST:PORT/PATH"
                            .into(),
                    )
                })?)
                .to_owned(),
                speed: speed as u32,
            }),
        };
        Ok(Self {
            device: settings.get("device").map(|value| (*value).to_owned()),
            host,
            select: select as u32,
            write_only,
        })
    }
}

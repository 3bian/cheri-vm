// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::parse_u32;
use super::super::socket::{Host, SocketOptions};
use super::UartFiles;
use crate::machine::EmulatorError;
use crate::machine::host::uart::settings::UartSettings;
use crate::runtime::device_settings;

pub(crate) struct UartOptions {
    pub device: Option<String>,
    pub host: Host<UartFiles>,
}

impl UartOptions {
    pub(crate) fn parse(value: &str) -> Result<Self, EmulatorError> {
        let settings = device_settings::parse(
            value,
            "--uart",
            &[
                "baud",
                "data-bits",
                "device",
                "flow",
                "in",
                "out",
                "parity",
                "stop-bits",
                "timeout",
                "ws",
            ],
        )?;
        let device = settings.get("device").map(|value| (*value).to_owned());
        if let Some(socket) = SocketOptions::parse(
            &settings,
            "--uart",
            &["in", "out"],
            &["baud", "data-bits", "flow", "parity", "stop-bits"],
        )? {
            return Ok(Self {
                device,
                host: Host::WebSocket(socket),
            });
        }
        if !settings.contains_key("in") && !settings.contains_key("out") {
            return Err(EmulatorError::Configuration(
                "--uart requires in=PATH, out=PATH, ws or ws://HOST:PORT/PATH"
                    .into(),
            ));
        }
        let framing = if settings.keys().any(|key| {
            matches!(
                *key,
                "baud" | "data-bits" | "flow" | "parity" | "stop-bits"
            )
        }) {
            let baud = settings
                .get("baud")
                .map(|value| parse_u32(value, "UART baud rate"))
                .transpose()?;
            let data_bits =
                settings.get("data-bits").map_or(Ok(8), |value| {
                    value.parse::<u8>().map_err(|_| {
                        EmulatorError::Configuration(
                            "invalid UART data-bits".into(),
                        )
                    })
                })?;
            let stop_bits =
                settings.get("stop-bits").map_or(Ok(1), |value| {
                    value.parse::<u8>().map_err(|_| {
                        EmulatorError::Configuration(
                            "invalid UART stop-bits".into(),
                        )
                    })
                })?;
            let framing = UartSettings {
                baud,
                data_bits,
                flow: settings
                    .get("flow")
                    .map(|value| value.parse())
                    .transpose()?
                    .unwrap_or_default(),
                parity: settings
                    .get("parity")
                    .map(|value| value.parse())
                    .transpose()?
                    .unwrap_or_default(),
                stop_bits,
            };
            framing.validate()?;
            Some(framing)
        } else {
            None
        };
        Ok(Self {
            device,
            host: Host::Linux(UartFiles {
                input: settings.get("in").map(|value| (*value).to_owned()),
                output: settings.get("out").map(|value| (*value).to_owned()),
                settings: framing,
            }),
        })
    }
}

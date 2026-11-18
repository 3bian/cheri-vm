// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::parse_u32;
use super::super::socket::{Host, SocketOptions};
use super::{GpioChip, pins};
use crate::machine::EmulatorError;
use crate::machine::host::gpio::settings::GpioSettings;
use crate::runtime::device_settings;

pub(crate) struct GpioOptions {
    pub device: Option<String>,
    pub host: Host<GpioChip>,
    pub pins: Option<u32>,
}

impl GpioOptions {
    pub(crate) fn parse(value: &str) -> Result<Self, EmulatorError> {
        let settings = device_settings::parse(
            value,
            "--gpio",
            &[
                "active-low",
                "bias",
                "device",
                "drive",
                "lines",
                "path",
                "pins",
                "timeout",
                "ws",
            ],
        )?;
        let device = settings.get("device").map(|value| (*value).to_owned());
        if let Some(socket) = SocketOptions::parse(
            &settings,
            "--gpio",
            &["path"],
            &["active-low", "bias", "drive", "lines"],
        )? {
            return Ok(Self {
                device,
                host: Host::WebSocket(socket),
                pins: settings
                    .get("pins")
                    .map(|value| pins(value))
                    .transpose()?,
            });
        }
        let (Some(path), Some(lines)) =
            (settings.get("path"), settings.get("lines"))
        else {
            return Err(EmulatorError::Configuration(
                "--gpio requires path=PATH and lines=OFFSET+OFFSET, ws or \
                 ws://HOST:PORT/PATH"
                    .into(),
            ));
        };
        let lines = lines
            .split('+')
            .map(|value| parse_u32(value, "GPIO line offset"))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            device,
            host: Host::Linux(GpioChip {
                lines,
                path: (*path).to_owned(),
                settings: GpioSettings {
                    active_low: settings
                        .get("active-low")
                        .map(|value| value.parse::<bool>())
                        .transpose()
                        .map_err(|_| {
                            EmulatorError::Configuration(
                                "GPIO active-low requires true or false".into(),
                            )
                        })?
                        .unwrap_or(false),
                    bias: match settings.get("bias") {
                        None | Some(&"as-is") => None,
                        Some(value) => Some(value.parse()?),
                    },
                    drive: settings
                        .get("drive")
                        .map(|value| value.parse())
                        .transpose()?
                        .unwrap_or_default(),
                },
            }),
            pins: None,
        })
    }
}

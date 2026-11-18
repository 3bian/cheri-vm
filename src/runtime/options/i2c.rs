// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::EmulatorError;

use super::parse_unsigned;
use super::socket::{Host, SocketOptions};
use crate::machine::devices::i2c::validate_host_addresses;
use crate::runtime::device_settings;

pub(crate) struct I2cOptions {
    pub addresses: Option<Vec<u8>>,
    pub device: Option<String>,
    pub host: Host<String>,
}

impl I2cOptions {
    pub(crate) fn parse(value: &str) -> Result<Self, EmulatorError> {
        let settings = device_settings::parse(
            value,
            "--i2c",
            &["addresses", "device", "path", "timeout", "ws"],
        )?;
        let addresses = settings
            .get("addresses")
            .map(|value| {
                let addresses = value
                    .split('+')
                    .map(|value| {
                        u8::try_from(parse_unsigned(value, "I2C address")?)
                            .map_err(|_| {
                                EmulatorError::Configuration(
                                    "I2C address exceeds seven bits".into(),
                                )
                            })
                    })
                    .collect::<Result<Vec<_>, EmulatorError>>()?;
                validate_host_addresses(&addresses)?;
                Ok::<_, EmulatorError>(addresses)
            })
            .transpose()?;
        let host = match SocketOptions::parse(
            &settings,
            "--i2c",
            &["path"],
            &[],
        )? {
            Some(_) if addresses.is_none() => {
                return Err(EmulatorError::Configuration(
                    "--i2c on a WebSocket needs addresses=A+A, one address for each"
                        .into(),
                ));
            }
            Some(socket) => Host::WebSocket(socket),
            None => Host::Linux(
                (*settings.get("path").ok_or_else(|| {
                    EmulatorError::Configuration(
                        "--i2c requires path=PATH, ws or ws://HOST:PORT/PATH"
                            .into(),
                    )
                })?)
                .to_owned(),
            ),
        };
        Ok(Self {
            addresses,
            device: settings.get("device").map(|value| (*value).to_owned()),
            host,
        })
    }
}

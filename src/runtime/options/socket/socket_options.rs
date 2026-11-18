// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::parse_unsigned;
use super::parts;
use crate::machine::EmulatorError;
use crate::machine::host::websocket::Address;
use crate::runtime::device_settings::WEB_SOCKET;
use std::collections::HashMap;
use std::time::Duration;

pub(crate) struct SocketOptions {
    pub address: Address,
    pub timeout: Duration,
}

impl SocketOptions {
    pub(in crate::runtime::options) fn parse(
        settings: &HashMap<&str, &str>,
        flag: &str,
        linux_keys: &[&str],
        linux_only: &[&str],
    ) -> Result<Option<Self>, EmulatorError> {
        let Some(text) = settings.get(WEB_SOCKET) else {
            if let Some(key) = ["pins", "timeout"]
                .into_iter()
                .find(|key| settings.contains_key(key))
            {
                return Err(EmulatorError::Configuration(format!(
                    "{flag} {key} needs a WebSocket: ws or \
                     ws://HOST:PORT/PATH"
                )));
            }
            return Ok(None);
        };
        if linux_keys.iter().any(|key| settings.contains_key(key)) {
            return Err(EmulatorError::Configuration(format!(
                "{flag} takes {} or a WebSocket, not both; a WebSocket \
                 carries both directions",
                linux_keys.join(" and ")
            )));
        }
        if let Some(key) =
            linux_only.iter().find(|key| settings.contains_key(*key))
        {
            return Err(EmulatorError::Configuration(format!(
                "{flag} {key} applies to {}, not to a WebSocket",
                linux_keys.join(" and ")
            )));
        }
        let address = if *text == "ws" {
            Address::Served
        } else {
            parts(text)?;
            Address::Remote((*text).to_owned())
        };
        let timeout = settings
            .get("timeout")
            .map_or(Ok(1_000), |value| parse_unsigned(value, "timeout"))?;
        if !(1..=60_000).contains(&timeout) {
            return Err(EmulatorError::Configuration(
                "timeout must be 1 to 60000 milliseconds".into(),
            ));
        }
        Ok(Some(Self {
            address,
            timeout: Duration::from_millis(timeout),
        }))
    }
}

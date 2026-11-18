// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use crate::machine::EmulatorError;

pub(crate) const WEB_SOCKET: &str = "ws";

pub(super) fn parse<'a>(
    value: &'a str,
    flag: &str,
    allowed: &[&str],
) -> Result<HashMap<&'a str, &'a str>, EmulatorError> {
    let mut settings = HashMap::new();
    for (index, setting) in value.split(',').enumerate() {
        let names_socket = index == 0
            && !setting.contains('=')
            && (setting == WEB_SOCKET
                || setting.starts_with("ws:")
                || setting.starts_with("wss:"));
        if names_socket && allowed.contains(&WEB_SOCKET) {
            settings.insert(WEB_SOCKET, setting);
            continue;
        }
        let (key, value) = setting.split_once('=').ok_or_else(|| {
            EmulatorError::Configuration(format!(
                "{flag} takes KEY=VALUE settings separated by commas, not \
                 {setting:?}; see --help for the keys"
            ))
        })?;
        if !allowed.contains(&key) || key == WEB_SOCKET {
            return Err(EmulatorError::Configuration(format!(
                "unknown {flag} setting {key:?}; the settings are {}",
                allowed
                    .iter()
                    .filter(|key| **key != WEB_SOCKET)
                    .copied()
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        if value.is_empty() {
            return Err(EmulatorError::Configuration(format!(
                "{flag} {key} must not be empty"
            )));
        }
        if settings.insert(key, value).is_some() {
            return Err(EmulatorError::Configuration(format!(
                "{flag} {key} may only be supplied once"
            )));
        }
    }
    Ok(settings)
}

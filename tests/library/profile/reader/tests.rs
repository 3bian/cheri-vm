// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::profile::reader::{EmulatorSettings, MemoryEntry, ProfileDevice};

#[test]
fn every_emulator_key_is_named_in_the_page_of_profile_keys() {
    let Ok(text) = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../book/appendix-b-profile-keys.md"
    )) else {
        return;
    };
    let mut missing = Vec::new();
    for keys in [
        keys_of::<EmulatorSettings>(),
        keys_of::<MemoryEntry>(),
        keys_of::<ProfileDevice>(),
    ] {
        assert!(!keys.is_empty());
        missing.extend(
            keys.into_iter()
                .filter(|key| !text.contains(&format!("`{key}`"))),
        );
    }
    assert!(missing.is_empty(), "{missing:?}");
}

fn keys_of<T: serde::de::DeserializeOwned>() -> Vec<String> {
    let message = match toml::from_str::<T>("__probe = 1") {
        Ok(_) => return Vec::new(),
        Err(error) => error.to_string(),
    };
    message
        .split_once("expected")
        .map(|(_, expected)| {
            expected
                .split('`')
                .skip(1)
                .step_by(2)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

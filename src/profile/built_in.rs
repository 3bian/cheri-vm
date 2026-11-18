// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

pub const FOLDER: &str = "<built in>";

pub const MACHINES: &[(&str, &str)] = &[
    ("cheriot", include_str!("../../profiles/cheriot.toml")),
    (
        "cheriot-hwr",
        include_str!("../../profiles/cheriot-hwr.toml"),
    ),
    ("rv32e", include_str!("../../profiles/rv32e.toml")),
    ("rv32imc", include_str!("../../profiles/rv32imc.toml")),
    ("rv64imac", include_str!("../../profiles/rv64imac.toml")),
    ("cva6", include_str!("../../profiles/cva6.toml")),
    ("cva6-cheri", include_str!("../../profiles/cva6-cheri.toml")),
];

pub fn names() -> Vec<String> {
    MACHINES
        .iter()
        .map(|(name, _)| (*name).to_owned())
        .collect()
}

pub fn path(name: &str) -> PathBuf {
    PathBuf::from(FOLDER).join(format!("{name}.toml"))
}

pub fn text(name: &str) -> Option<&'static str> {
    MACHINES
        .iter()
        .find(|(listed, _)| *listed == name)
        .map(|(_, text)| *text)
}

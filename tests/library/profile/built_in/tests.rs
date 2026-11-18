// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::profile::built_in::{names, path, text};
use crate::profile::load;
use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn every_machine_loads_by_name_and_the_hardware_revoker_uses_its_device() {
    for name in names() {
        let document = load(&name).unwrap();
        assert_eq!(document.name, name);
        assert_eq!(document.file, path(&name));
    }
    let revoker = load("cheriot-hwr").unwrap();
    assert_eq!(
        revoker.table["device"]["revoker"]["model"].as_str(),
        Some("revoker")
    );
    assert!(
        revoker.table["device"]["revoker"]
            .get("interrupt_source")
            .is_none()
    );
}

fn folder() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/profiles"))
}

#[test]
fn the_list_of_built_in_machines_is_the_profiles_folder() {
    let files: BTreeSet<String> = std::fs::read_dir(folder())
        .unwrap()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension()? == "toml")
                .then(|| path.file_stem()?.to_str().map(str::to_owned))?
        })
        .collect();
    let listed: BTreeSet<String> = names().into_iter().collect();
    assert_eq!(listed, files);
    for name in names() {
        let on_disk =
            std::fs::read_to_string(folder().join(format!("{name}.toml")))
                .unwrap();
        assert_eq!(text(&name).unwrap(), on_disk, "{name}");
    }
}

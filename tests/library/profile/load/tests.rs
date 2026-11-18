// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::profile::{built_in, load};
use crate::program::temporary::TemporaryDirectory;

fn written(folder: &TemporaryDirectory, name: &str, text: &str) -> String {
    let file = folder.path().join(name);
    std::fs::write(&file, text).unwrap();
    file.to_str().unwrap().to_owned()
}

#[test]
fn a_device_that_names_an_interrupt_no_table_defines_is_refused() {
    let folder = TemporaryDirectory::new().unwrap();
    let file = written(
        &folder,
        "odd.toml",
        "name = \"odd\"\n[device.spi]\nstart = 4096\nmodel = \"spi\"\n\
         interrupt = \"spi\"\n",
    );
    let error = load(&file).unwrap_err();
    assert!(
        error.message.ends_with(
            "device spi names the interrupt spi, which [interrupt] does not \
             define"
        ),
        "{error}"
    );
}

#[test]
fn a_device_with_emulator_keys_needs_a_model() {
    let folder = TemporaryDirectory::new().unwrap();
    let file = written(
        &folder,
        "odd.toml",
        "name = \"odd\"\n[device.disk]\nstart = 4096\nsectors = 8\n",
    );
    let error = load(&file).unwrap_err();
    assert!(
        error
            .message
            .ends_with("device disk has emulator keys but no model"),
        "{error}"
    );
}

#[test]
fn a_file_extends_a_built_in_profile_and_removes_keys_from_it() {
    let folder = TemporaryDirectory::new().unwrap();
    let file = written(
        &folder,
        "small.toml",
        "name = \"small\"\nextends = \"rv32imc\"\n\
         remove = [\"device/ethernet\", \"device/storage/sectors\"]\n\
         [device.uart]\ninterrupt_source = 9\n",
    );
    let profile = load(&file).unwrap();
    assert_eq!(profile.name, "small");
    let devices = profile.table["device"].as_table().unwrap();
    assert!(!devices.contains_key("ethernet"));
    assert!(devices["storage"].get("sectors").is_none());
    assert_eq!(devices["storage"]["read_only"].as_bool(), Some(false));
    assert_eq!(devices["uart"]["interrupt_source"].as_integer(), Some(9));
    assert_eq!(devices["uart"]["model"].as_str(), Some("uart"));
    assert!(profile.has_machine());
}

#[test]
fn a_file_that_extends_a_file_beside_it_takes_that_one_before_a_built_in() {
    let folder = TemporaryDirectory::new().unwrap();
    written(
        &folder,
        "cheriot.toml",
        "name = \"local\"\ndescription = \"beside\"\n",
    );
    let file = written(
        &folder,
        "child.toml",
        "name = \"child\"\nextends = \"cheriot\"\n",
    );
    let profile = load(&file).unwrap();
    assert_eq!(profile.description.as_deref(), Some("beside"));
    assert!(!profile.has_machine());
}

#[test]
fn a_missing_file_and_an_unknown_name_list_the_profiles_there_are() {
    for selector in ["./nowhere.toml", "nowhere"] {
        let error = load(selector).unwrap_err();
        assert!(error.message.contains("rv32imc"), "{error}");
    }
    let unknown = load("nowhere").unwrap_err();
    assert!(unknown.message.starts_with("unknown profile"), "{unknown}");
}

#[test]
fn a_profile_that_extends_itself_is_a_cycle() {
    let folder = TemporaryDirectory::new().unwrap();
    let file = written(
        &folder,
        "loop.toml",
        "name = \"loop\"\nextends = \"loop\"\n",
    );
    let error = load(&file).unwrap_err();
    assert!(
        error.message.starts_with("profile inheritance cycle at"),
        "{error}"
    );
}

#[test]
fn every_built_in_profile_loads_under_its_own_name_and_has_a_machine() {
    assert_eq!(built_in::names().len(), 7);
    for name in built_in::names() {
        let profile = load(&name).unwrap();
        assert_eq!(profile.name, name);
        assert!(profile.has_machine(), "{name}");
        assert!(profile.description.is_some(), "{name}");
        assert!(profile.to_toml().unwrap().contains("[emulator]"), "{name}");
    }
}

#[test]
fn extends_takes_a_name_and_remove_names_keys_that_are_there() {
    let folder = TemporaryDirectory::new().unwrap();
    let path = written(
        &folder,
        "path.toml",
        "name = \"path\"\nextends = \"./rv32imc.toml\"\n",
    );
    let error = load(&path).unwrap_err();
    assert!(
        error.message.contains("extends takes a profile name"),
        "{error}"
    );
    let absent = written(
        &folder,
        "absent.toml",
        "name = \"absent\"\nextends = \"rv32imc\"\nremove = [\"device/none\"]\n",
    );
    let error = load(&absent).unwrap_err();
    assert!(
        error
            .message
            .ends_with("remove names device/none, which is not present"),
        "{error}"
    );
}

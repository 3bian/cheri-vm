// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::plain::model::Flat;
use crate::hart::privilege::Supervisor;
use crate::hart::{Cva6, Cva6Cheri, Hart, Rv64Plain, Rv64Rvy};
use crate::isa::extension::ExtensionSet;
use crate::machine::EmulatorError;
use crate::machine::configuration::*;
use crate::machine::memory::Tagging;

const BASE: &str = "name = \"demo\"\n[emulator]\nprocessor = \
                    \"rv32\"\nextensions = [\"i\", \"m\", \"c\", \"zicsr\", \
                    \"zifencei\"]\n[emulator.memory.ram]\nstart = 0\nlength = \
                    4096\n";

impl MachineConfiguration {
    pub(crate) fn build_capability_raw(
        self,
        address: u64,
        firmware: &[u8],
    ) -> Result<Hart<Rv64Rvy>, EmulatorError> {
        self.build_wide_machine(false, address, firmware)
    }

    pub(crate) fn build_cheri_supervisor_raw(
        self,
        address: u64,
        firmware: &[u8],
    ) -> Result<Hart<Cva6Cheri>, EmulatorError> {
        self.build_wide_machine(false, address, firmware)
    }

    pub(crate) fn build_supervisor_elf(
        self,
        firmware: &[u8],
    ) -> Result<Hart<Cva6>, EmulatorError> {
        self.build_wide_machine(true, 0, firmware)
    }

    pub(crate) fn build_supervisor_raw(
        self,
        address: u64,
        firmware: &[u8],
    ) -> Result<Hart<Cva6>, EmulatorError> {
        self.build_wide_machine(false, address, firmware)
    }

    pub(crate) fn build_wide_elf(
        self,
        firmware: &[u8],
    ) -> Result<Hart<Rv64Plain>, EmulatorError> {
        self.build_wide_machine(true, 0, firmware)
    }

    pub(crate) fn build_wide_raw(
        self,
        address: u64,
        firmware: &[u8],
    ) -> Result<Hart<Rv64Plain>, EmulatorError> {
        self.build_wide_machine(false, address, firmware)
    }
}

#[test]
fn a_cheriot_processor_has_one_set_of_extensions() {
    let cheriot = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/profiles/cheriot.toml"
    ));
    let fixed = crate::hart::cheriot_rv32e::Processor::EXTENSIONS;
    assert_eq!(
        crate::profile::reader::read(cheriot).unwrap().extensions,
        fixed
    );
    assert_eq!(
        fixed.names().collect::<Vec<_>>(),
        ["cheriot", "e", "m", "c", "zicsr", "zifencei"]
    );

    let same = cheriot.replace(
        "processor = \"cheriot\"",
        "processor = \"cheriot\"\nextensions = [\"cheriot\", \"e\", \"m\", \
         \"c\", \"zicsr\", \"zifencei\"]",
    );
    assert_eq!(
        crate::profile::reader::read(&same).unwrap().extensions,
        fixed
    );
    let fewer = cheriot.replace(
        "processor = \"cheriot\"",
        "processor = \"cheriot\"\nextensions = [\"cheriot\", \"e\"]",
    );
    let message = crate::profile::reader::read(&fewer)
        .unwrap_err()
        .to_string();
    assert!(message.contains("one set of extensions"), "{message}");
}

#[test]
fn a_machine_holds_up_to_two_gibibytes_in_all() {
    let with = |regions: usize| {
        let memory = (0..regions)
            .map(|index| {
                format!(
                    "ram{index} = {{ start = {:#x}, length = 0x4000_0000 }}",
                    0x1_0000_0000_u64 + (index as u64) * 0x4000_0000
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        crate::profile::reader::read(&format!(
            "name = \"large\"\n[emulator]\nprocessor = \"rv64\"\nextensions = \
             [\"i\", \"zicsr\"]\ntiming = \"per-step\"\nmemory = {{ {memory} \
             }}\n"
        ))
        .and_then(|configuration| {
            configuration
                .build_wide_raw(0x1_0000_0000, &[0x13, 0, 0, 0])
                .map(|_| ())
        })
    };
    assert!(with(2).is_ok());
    let message = with(3).unwrap_err().to_string();
    assert!(message.contains("2 GiB"), "{message}");
}

#[test]
fn a_parsed_configuration_builds_a_machine() {
    let config = crate::profile::reader::read(BASE).unwrap();
    assert!(config.extensions.contains(ExtensionSet::C));
    assert_eq!(config.build(0).unwrap().bus.memory.len(), 1);
}

#[test]
fn a_profile_lists_its_extensions_by_name() {
    let refused = |text: &str| {
        crate::profile::reader::read(text).unwrap_err().to_string()
    };
    let listed = "extensions = [\"i\", \"m\", \"c\", \"zicsr\", \"zifencei\"]";

    let config = crate::profile::reader::read(
        &BASE.replace(listed, "extensions = [\"e\"]"),
    )
    .unwrap();
    assert_eq!(config.extensions, ExtensionSet::E);

    let message = refused(&BASE.replace(&format!("{listed}\n"), ""));
    assert!(message.contains("needs an extensions list"), "{message}");
    let message = refused(&BASE.replace(listed, "extensions = [\"i\", \"q\"]"));
    assert!(message.contains("extension q is unknown"), "{message}");
    let message = refused(&BASE.replace(listed, "extensions = [\"i\", \"e\"]"));
    assert!(message.contains("one of i and e"), "{message}");
    let message = refused(&BASE.replace(listed, "extensions = [\"m\"]"));
    assert!(message.contains("one of i and e"), "{message}");
    let message =
        refused(&BASE.replace(listed, "extensions = [\"e\", \"cheriot\"]"));
    assert!(message.contains("processor = \"cheriot\""), "{message}");
    let message = refused(&BASE.replace("\"rv32\"", "\"rv32imc\""));
    assert!(
        message.contains("list its letters in extensions"),
        "{message}"
    );
}

#[test]
fn a_revoker_belongs_to_a_cheriot_machine() {
    let text = format!(
        "{BASE}\n[device.revoker]\nstart = 0x10004000\nmodel = \"revoker\"\n"
    );
    let error = crate::profile::reader::read(&text)
        .unwrap()
        .build(0)
        .err()
        .unwrap()
        .to_string();
    assert!(
        error.contains("the revoker model needs a CHERIoT processor"),
        "{error}"
    );

    let shipped = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/profiles/cheriot.toml"
    ));
    let system = crate::profile::reader::read(shipped)
        .unwrap()
        .build_cheriot_raw(0x8000_0000, &[0x13, 0, 0, 0])
        .unwrap();
    let revoker = system
        .bus
        .devices()
        .iter()
        .find(|mapping| mapping.name == "revoker")
        .unwrap();
    assert_eq!(
        (revoker.base, revoker.interrupt_source),
        (0x1000_4000, Some(7))
    );
}

#[test]
fn an_interrupt_source_is_one_its_controller_has() {
    let cva6 = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/profiles/cva6.toml"
    ));
    for (source, accepted) in [(1, true), (30, true), (31, false)] {
        let changed = cva6.replace(
            "interrupt_source = 1 ",
            &format!("interrupt_source = {source} "),
        );
        assert!(changed.contains(&format!("interrupt_source = {source} ")));
        let built = crate::profile::reader::read(&changed)
            .unwrap()
            .build_wide_machine::<Supervisor, Flat>(
                false,
                0x8000_0000,
                &[0x13, 0, 0, 0],
            );
        match built {
            Ok(_) => assert!(accepted, "{source}"),
            Err(error) => {
                assert!(!accepted, "{source}: {error}");
                assert!(error.to_string().contains("1..30"), "{error}");
            }
        }
    }

    let rv32imc = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/profiles/rv32imc.toml"
    ));
    let changed =
        rv32imc.replace("interrupt_source = 6", "interrupt_source = 31");
    assert_ne!(changed, rv32imc);
    assert!(
        crate::profile::reader::read(&changed)
            .unwrap()
            .build_raw(0x8000_0000, &[0x13, 0, 0, 0])
            .is_ok()
    );
}

#[test]
fn an_rv64_processor_has_32_registers_and_builds_only_the_64_bit_machine() {
    let wide = BASE.replace("\"rv32\"", "\"rv64\"");
    let config = crate::profile::reader::read(&wide).unwrap();
    assert!(config.is_wide());
    assert!(config.build_wide_raw(0, &[0x13, 0, 0, 0]).is_ok());
    let message = crate::profile::reader::read(&wide)
        .unwrap()
        .build(0)
        .err()
        .unwrap()
        .to_string();
    assert!(message.contains("not a 32-bit one"), "{message}");
    let message = crate::profile::reader::read(BASE)
        .unwrap()
        .build_wide_raw(0, &[0x13, 0, 0, 0])
        .err()
        .unwrap()
        .to_string();
    assert!(message.contains("not a 64-bit one"), "{message}");

    let embedded = wide.replace("[\"i\", ", "[\"e\", ");
    let message = crate::profile::reader::read(&embedded)
        .unwrap_err()
        .to_string();
    assert!(message.contains("list i, and not e"), "{message}");
    let message =
        crate::profile::reader::read(&BASE.replace("\"rv32\"", "\"rv64gc\""))
            .unwrap_err()
            .to_string();
    assert!(message.contains("processor = \"rv64\""), "{message}");
    let atomic = BASE.replace("\"m\", ", "\"m\", \"a\", ");
    assert!(
        crate::profile::reader::read(&atomic)
            .unwrap()
            .extensions
            .contains(ExtensionSet::A)
    );
}

#[test]
fn an_unsupported_architecture_or_overlapping_memory_is_refused() {
    let text = BASE.replace("\"rv32\"", "\"minimal-cheriot\"");
    assert!(crate::profile::reader::read(&text).is_err());
    let text = format!(
        "{BASE}\n[emulator.memory.overlap]\ntype = \"rom\"\nstart = \
         2048\nlength = 1024\n"
    );
    assert!(
        crate::profile::reader::read(&text)
            .unwrap()
            .build(0)
            .is_err()
    );
}

#[test]
fn capabilities_are_for_a_64_bit_machine_in_machine_mode_and_tag_its_memory() {
    let with = |processor: &str, extensions: &str| {
        crate::profile::reader::read(&format!(
            "name = \"capability\"\n[emulator]\nprocessor = \
             \"{processor}\"\nextensions = {extensions}\ntiming = \
             \"per-step\"\nmemory = {{ ram = {{ start = 0x8000_0000, length = \
             0x1000 }}, rom = {{ start = 0x1000, length = 0x1000, tagged = \
             false }} }}\n"
        ))
    };
    let machine = with("rv64", "[\"y\", \"i\", \"zicsr\"]")
        .unwrap()
        .build_capability_raw(0x8000_0000, &[0x13, 0, 0, 0])
        .unwrap();
    let tagging = |name: &str| {
        machine
            .bus
            .memory
            .iter()
            .find(|region| region.name == name)
            .unwrap()
            .tagging()
    };
    assert_eq!(tagging("ram"), Tagging::Rvy);
    assert_eq!(tagging("rom"), Tagging::None);
    for (processor, extensions, expected) in [
        (
            "rv32",
            "[\"y\", \"i\", \"zicsr\"]",
            "y needs processor = \"rv64\"",
        ),
        ("rv64", "[\"y\", \"i\"]", "y needs zicsr"),
    ] {
        let message = with(processor, extensions).unwrap_err().to_string();
        assert!(message.contains(expected), "{message}");
    }
    let message = with("rv64", "[\"y\", \"i\", \"zicsr\"]")
        .unwrap()
        .build_wide_raw(0x8000_0000, &[0x13, 0, 0, 0])
        .err()
        .unwrap()
        .to_string();
    assert!(message.contains("has capabilities"), "{message}");
}

#[test]
fn double_precision_needs_single_and_floating_point_needs_32_registers() {
    let with = |extensions: &str| {
        crate::profile::reader::read(&BASE.replace(
            "[\"i\", \"m\", \"c\", \"zicsr\", \"zifencei\"]",
            extensions,
        ))
    };
    assert!(with("[\"i\", \"f\", \"d\", \"zicsr\"]").is_ok());
    assert!(with("[\"i\", \"f\", \"zicsr\"]").is_ok());
    for (extensions, expected) in [
        ("[\"i\", \"d\", \"zicsr\"]", "d needs f"),
        ("[\"e\", \"f\", \"zicsr\"]", "f needs i and zicsr"),
        ("[\"i\", \"f\"]", "f needs i and zicsr"),
    ] {
        let message = with(extensions).unwrap_err().to_string();
        assert!(message.contains(expected), "{message}");
    }
}

#[test]
fn every_extension_is_named_in_the_page_of_profile_keys() {
    let Ok(page) = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../book/appendix-b-profile-keys.md"
    )) else {
        return;
    };
    for extension in crate::isa::extension::EXTENSIONS.iter() {
        let row = format!("| `{}` | ", extension.name);
        assert!(page.contains(&row), "the page of profile keys lacks {row}");
    }
}

#[test]
fn invalid_group_is_rejected() {
    let text = BASE.replace("[emulator]", "[emulator]\nno_such_key = true");
    assert!(crate::profile::reader::read(&text).is_err());
}

#[test]
fn supervisor_and_user_modes_come_together_on_a_64_bit_machine() {
    let modes = |processor: &str, extensions: &str| {
        crate::profile::reader::read(
            &BASE
                .replace("\"rv32\"", &format!("\"{processor}\""))
                .replace(
                    "\"zifencei\"]",
                    &format!("\"zifencei\"{extensions}]"),
                ),
        )
    };
    let config = modes("rv64", ", \"s\", \"u\"").unwrap();
    assert!(config.is_supervisor());
    assert!(config.build_supervisor_raw(0, &[0x13, 0, 0, 0]).is_ok());
    let message = modes("rv64", ", \"s\", \"u\"")
        .unwrap()
        .build_wide_raw(0, &[0x13, 0, 0, 0])
        .err()
        .unwrap()
        .to_string();
    assert!(
        message.contains("has supervisor and user modes"),
        "{message}"
    );
    for (processor, extensions, expected) in [
        ("rv64", ", \"s\"", "list s and u together"),
        ("rv64", ", \"u\"", "list s and u together"),
        ("rv32", ", \"s\", \"u\"", "need processor = \"rv64\""),
    ] {
        let message = modes(processor, extensions).unwrap_err().to_string();
        assert!(message.contains(expected), "{message}");
    }
}

#[test]
fn the_short_forms_of_zcb_need_the_extensions_they_stand_for() {
    let with = |extensions: &str| {
        crate::profile::reader::read(&BASE.replace(
            "[\"i\", \"m\", \"c\", \"zicsr\", \"zifencei\"]",
            extensions,
        ))
    };
    assert!(with("[\"i\", \"m\", \"c\", \"zba\", \"zbb\", \"zcb\"]").is_ok());
    assert!(with("[\"e\", \"zbs\", \"zbc\", \"zicond\", \"zkn\"]").is_ok());
    for extensions in [
        "[\"i\", \"m\", \"zba\", \"zbb\", \"zcb\"]",
        "[\"i\", \"c\", \"zba\", \"zbb\", \"zcb\"]",
        "[\"i\", \"m\", \"c\", \"zbb\", \"zcb\"]",
        "[\"i\", \"m\", \"c\", \"zba\", \"zcb\"]",
    ] {
        let message = with(extensions).unwrap_err().to_string();
        assert!(message.contains("zcb needs c, m, zba and zbb"), "{message}");
    }
}

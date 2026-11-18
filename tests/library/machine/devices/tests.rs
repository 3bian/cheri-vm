// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::configuration::DeviceConfiguration;
use crate::machine::devices::{
    DEVICE_KINDS, ethernet, gpio, i2c, interrupt_controller, spi, storage,
    timer64,
};

macro_rules! map {
    ($register:ty) => {
        <$register>::ALL
            .iter()
            .map(|&(register, name)| (register.offset(), name))
            .collect::<Vec<_>>()
    };
}

const SDK_HEADERS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../integrations/cheriot-rtos/sdk/include/platform/cheri-vm"
);

fn compare_with_sdk(header: &str, registers: &[(u32, &str)]) {
    let Some(text) = sdk_header(header) else {
        return;
    };
    let names: Vec<&str> = registers.iter().map(|&(_, name)| name).collect();
    assert_eq!(header_fields(&text), names, "{header}");
    for (index, &(offset, name)) in registers.iter().enumerate() {
        assert_eq!(offset as usize, index * 4, "{header} {name}");
    }
    let assertions = header_assertions(&text);
    assert!(!assertions.is_empty(), "{header}");
    for (field, value) in assertions {
        match field {
            None => {
                assert_eq!(value as usize, registers.len() * 4, "{header} size")
            }
            Some(field) => {
                let offset = registers
                    .iter()
                    .find(|&&(_, name)| name == field)
                    .unwrap()
                    .0;
                assert_eq!(offset, value, "{header} {field}");
            }
        }
    }
}

#[test]
fn every_device_rejects_invalid_widths_and_offsets() {
    for kind in DEVICE_KINDS {
        let configuration = DeviceConfiguration {
            attached_device: "none".into(),
            base: 0,
            interrupt_source: None,
            kind: kind.name.into(),
            layout: None,
            name: kind.name.into(),
            read_only: false,
            sectors: 1,
        };
        let mut device = (kind.make)(&configuration).unwrap();
        for width in [0, 1, 2, 3, 8, usize::MAX] {
            assert!(device.read(0, width).is_err(), "{}", kind.name);
            assert!(device.write(0, width, 0).is_err(), "{}", kind.name);
        }
        for offset in [1, 2, 3, kind.size, u32::MAX] {
            assert!(
                device.read(offset, 4).is_err(),
                "{} at {offset:#x}",
                kind.name
            );
            assert!(
                device.write(offset, 4, 0).is_err(),
                "{} at {offset:#x}",
                kind.name
            );
        }
    }
}

#[test]
fn every_register_map_matches_its_sdk_header() {
    compare_with_sdk("platform-gpio.hh", &map!(gpio::registers::Register));
    compare_with_sdk("platform-spi.hh", &map!(spi::registers::Register));
    compare_with_sdk("platform-i2c.hh", &map!(i2c::registers::Register));
    compare_with_sdk(
        "platform-storage.hh",
        &map!(storage::registers::Register),
    );
    compare_with_sdk(
        "platform-ethernet.hh",
        &map!(ethernet::registers::Register),
    );
    compare_with_sdk(
        "platform-plic.hh",
        &map!(interrupt_controller::registers::Register),
    );
}

fn header_assertions(header: &str) -> Vec<(Option<String>, u32)> {
    header
        .lines()
        .filter_map(|line| {
            let inner = line
                .trim()
                .strip_prefix("static_assert(")?
                .strip_suffix(");")?;
            let (left, right) = inner.split_once(" == ")?;
            let value = right.trim().parse().ok()?;
            if left.starts_with("sizeof(") {
                Some((None, value))
            } else {
                let field =
                    left.strip_prefix("offsetof(")?.strip_suffix(')')?;
                Some((Some(field.split(", ").nth(1)?.to_owned()), value))
            }
        })
        .collect()
}

fn header_fields(header: &str) -> Vec<String> {
    let mut fields = Vec::new();
    for line in header.lines().map(str::trim) {
        let declaration = line.strip_prefix("const ").unwrap_or(line);
        let field = declaration
            .strip_prefix("uint32_t ")
            .and_then(|rest| rest.strip_suffix(';'))
            .filter(|name| {
                name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            });
        match field {
            Some(name) => fields.push(name.to_owned()),
            None if fields.is_empty() || line.is_empty() => {}
            None => break,
        }
    }
    fields
}

fn sdk_header(name: &str) -> Option<String> {
    std::fs::read_to_string(format!("{SDK_HEADERS}/{name}")).ok()
}

#[test]
fn the_timer_register_map_matches_the_sdk_time_read() {
    use timer64::registers::Register;

    let Some(header) = sdk_header("platform-time.h") else {
        return;
    };
    assert!(
        header.contains("high = timer[1];")
            && header.contains("low  = timer[0];")
    );
    assert_eq!(Register::TimeHigh.offset(), Register::TimeLow.offset() + 4);
    let profile: toml::Value = toml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/profiles/cheriot.toml"
    )))
    .unwrap();
    let start =
        |name: &str| profile["device"][name]["start"].as_integer().unwrap();
    assert_eq!(
        start("clint_time") - start("clint"),
        i64::from(Register::TimeLow.offset())
    );
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::debug::adapter::{Advanced, System, Trapped};
use crate::hart::execution::TrapPolicy;
use crate::hart::privilege::{MachineOnly, Supervisor};
use crate::isa::pointer::Rvy;
use crate::isa::pointer::rvy::Capability128;
use crate::machine::MachineConfiguration;
use crate::machine::TimingPolicy;

const BASE: u64 = 0x8000_0000;
const EBREAK: u32 = 0x0010_0073;
const LINK: u32 = 0x0080_00ef;

#[test]
fn a_capability_machine_describes_whole_capabilities() {
    let mut hart =
        configuration("rv64", "'y','i','m','a','c','zicsr','zifencei'")
            .build_wide_machine::<MachineOnly, Rvy>(
                false,
                BASE,
                &bytes(&[LINK, 0x0000_0013]),
            )
            .unwrap();
    hart.processor.set_capability(
        10,
        Capability128::INFINITE
            .with_address(BASE + 0x40)
            .with_bounds(16, true),
    );
    hart.processor
        .set_capability(1, Capability128::integer(BASE));
    let system: &mut dyn System = &mut hart;
    assert_eq!(system.return_address(), None);
    let registers = system.registers();
    assert_eq!(registers.len(), 33);
    assert!(
        registers[0]
            .value
            .starts_with("0x0000000080000000 (tag=1 base=0x0000000000000000 "),
        "{}",
        registers[0].value
    );
    assert_eq!(registers[11].name, "a0");
    assert!(
        registers[11].value.starts_with(
            "0x0000000080000040 (tag=1 base=0x0000000080000040 top=0x80000050 "
        ),
        "{}",
        registers[11].value
    );
    assert_eq!(registers[11].address, Some(BASE + 0x40));
    assert_eq!(system.advance().unwrap(), Advanced::default());
    assert_eq!(system.return_address(), Some(BASE + 4));
    let link = &system.registers()[2].value;
    assert!(link.ends_with(" sealed)"), "{link}");
    let described = system.monitor(&["capability", "x10"]).unwrap().unwrap();
    assert!(described.starts_with("x10: metadata=0x"), "{described}");
}

#[test]
fn a_plain_machine_describes_integer_and_floating_point_registers() {
    let mut narrow = configuration("rv32", "'i','m','c','zicsr','zifencei'")
        .build_raw(BASE, &bytes(&[LINK, 0x0000_0013, EBREAK]))
        .unwrap();
    narrow.trap_policy = TrapPolicy::Vector;
    let system: &mut dyn System = &mut narrow;
    assert_eq!(system.digits(), 8);
    assert_eq!(system.thread(), "RISC-V core");
    let registers = system.registers();
    assert_eq!(registers.len(), 33);
    assert_eq!(registers[0].value, "0x80000000");
    assert_eq!(registers[3].name, "sp");
    assert_eq!(registers[32].name, "t6");
    assert!(system.monitor(&["capability", "pcc"]).is_none());
    assert_eq!(system.advance().unwrap(), Advanced::default());
    assert_eq!(system.registers()[2].value, "0x80000004");
    assert_eq!(system.return_address(), Some(BASE + 4));
    assert_eq!(
        system.advance().unwrap().trap,
        Some(Trapped {
            address: BASE + 8,
            cause: 3,
            interrupt: false
        })
    );

    let mut wide = configuration(
        "rv64",
        "'i','m','a','f','d','c','zicsr','zifencei','s','u'",
    )
    .build_wide_machine::<Supervisor, crate::hart::plain::Flat>(
        false,
        BASE,
        &bytes(&[LINK]),
    )
    .unwrap();
    wide.processor.float_registers[1] = 1.5_f64.to_bits();
    wide.processor.float_registers[2] =
        0xffff_ffff_0000_0000 | u64::from(2.5_f32.to_bits());
    wide.processor.float_control = 0x21;
    let system: &mut dyn System = &mut wide;
    assert_eq!(system.digits(), 16);
    let registers = system.registers();
    assert_eq!(registers.len(), 66);
    assert_eq!(registers[0].value, "0x0000000080000000");
    assert_eq!(registers[33].name, "f0");
    assert_eq!(registers[34].value, "0x3ff8000000000000 (1.5)");
    assert_eq!(registers[35].value, "0xffffffff40200000 (2.5, single)");
    assert_eq!(registers[65].name, "fcsr");
    assert_eq!(registers[65].value, "0x0000000000000021");
    assert!(system.inspect(BASE - 1, &mut [0; 2]).is_err());
}

fn bytes(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|word| word.to_le_bytes()).collect()
}

fn configuration(processor: &str, extensions: &str) -> MachineConfiguration {
    let mut configuration = crate::profile::reader::read(&format!(
        "name='adapter-test'\n[emulator]\nprocessor='{processor}'\n\
         extensions=[{extensions}]\nmemory={{ram={{start=0x80000000,\
         length=0x10000}}}}\n"
    ))
    .unwrap();
    configuration.timing = TimingPolicy::PerStep;
    configuration
}

#[test]
fn the_cheriot_machine_describes_its_registers_as_before() {
    let shipped = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/profiles/cheriot.toml"
    ));
    let mut hart = crate::profile::reader::read(shipped)
        .unwrap()
        .build_cheriot_raw(BASE, &bytes(&[LINK, 0x0000_0013, EBREAK]))
        .unwrap();
    let system: &mut dyn System = &mut hart;
    assert_eq!(system.digits(), 8);
    assert_eq!(system.thread(), "CHERIoT core");
    assert_eq!(system.program_counter(), BASE);
    assert_eq!(system.return_address(), None);
    let registers = system.registers();
    let names: Vec<&str> =
        registers.iter().map(|held| held.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "pc", "czero", "cra", "csp", "cgp", "ctp", "ct0", "ct1", "ct2",
            "cs0", "cs1", "ca0", "ca1", "ca2", "ca3", "ca4", "ca5"
        ]
    );
    assert_eq!(registers[0].value, "0x80000000");
    assert_eq!(registers[0].address, None);
    assert_eq!(
        registers[1].value,
        "0x00000000 (tag=0 base=0x00000000 top=0x0 perms=0x000)"
    );
    assert_eq!(registers[1].address, Some(0));
    assert_eq!(system.advance().unwrap(), Advanced::default());
    assert_eq!(system.return_address(), Some(BASE + 4));
    assert!(
        system.registers()[2]
            .value
            .starts_with("0x80000004 (tag=1 ")
    );
    let described = system.monitor(&["capability", "pcc"]).unwrap().unwrap();
    assert!(described.starts_with("pcc: payload=0x"), "{described}");
    assert!(system.monitor(&["nothing"]).is_none());
    let mut memory = [0; 4];
    system.inspect(BASE, &mut memory).unwrap();
    assert_eq!(memory, LINK.to_le_bytes());
}

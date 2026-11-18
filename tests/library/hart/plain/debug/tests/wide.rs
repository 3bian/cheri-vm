// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use gdbstub::arch::{RegId, Registers as GdbRegisters};
use gdbstub::target::ext::base::singlethread::{
    SingleThreadBase, SingleThreadResume, SingleThreadSingleStep,
};
use gdbstub::target::ext::breakpoints::{HwBreakpoint, SwBreakpoint};

use super::firmware_from_words;
use crate::debug::gdb::{BreakpointType, StopReason, Target};
use crate::hart::Hart;
use crate::hart::execution::TrapPolicy;
use crate::hart::plain::debug::gdb::GdbModel;
use crate::hart::plain::{Flat, Plain};
use crate::hart::privilege::{MachineOnly, PrivilegeModel, Supervisor};
use crate::isa::pointer::Rvy;
use crate::isa::pointer::rvy::{Capability128, permissions};
use crate::isa::xlen::Rv64;
use crate::machine::TimingPolicy;
use crate::machine::firmware::load_raw;

const BASE: u64 = 0x8000_0000;
const BOTH: &str = "'y','i','m','a','f','d','c','zicsr','zifencei','s','u'";
const CAPABILITY: &str = "'y','i','m','a','c','zicsr','zifencei'";
const FLOAT: &str = "'i','m','a','f','d','c','zicsr','zifencei','s','u'";
const LOOP: [u32; 2] = [0x0010_8093, 0xffdf_f06f];
const MRET: u32 = 0x3020_0073;
const PLAIN: &str = "'i','m','a','c','zicsr','zifencei'";

type RegisterId = crate::hart::plain::debug::gdb::RegisterId<Rv64>;

type Registers = crate::hart::plain::debug::gdb::Registers<Rv64>;

type Wide<P, M> = Target<Hart<Plain<Rv64, P, M>>>;

#[test]
fn a_fault_on_a_64_bit_machine_is_recorded_at_its_width() {
    let mut hart =
        target::<MachineOnly, Flat>(PLAIN, &[0xffff_ffff, 0x0000_0013])
            .into_core();
    hart.processor.machine_registers.trap_vector = BASE + 4;
    let mut target = Target::new(hart, Some(4));
    assert_eq!(target.monitor(b"fault").unwrap(), "no trap observed\n");
    target.step(None).unwrap();
    assert_eq!(target.advance().unwrap(), Some(StopReason::Fault(2)));
    assert_eq!(
        target.monitor(b"fault").unwrap(),
        "fault address=0x0000000080000000 cause=0x0000000000000002 \
         value=0x00000000ffffffff resulting_pc=0x0000000080000004\n"
    );
    assert!(target.monitor(b"capability pcc").is_err());
    assert!(target.monitor(b"help").unwrap().starts_with("help | fault"));
}

fn breaks_steps_and_reads<P: PrivilegeModel, M: GdbModel<Rv64>>(
    extensions: &str,
) {
    let mut target = target::<P, M>(extensions, &LOOP);
    let mut before = [0; 8];
    target.inspect(BASE, &mut before).unwrap();
    assert_eq!(before[..4], LOOP[0].to_le_bytes());
    assert!(matches!(target.add_sw_breakpoint(BASE, 4), Ok(true)));
    target.resume(None).unwrap();
    assert_eq!(
        target.advance().unwrap(),
        Some(StopReason::Breakpoint(BASE, BreakpointType::Software))
    );
    assert_eq!(target.core().attempted_steps, 0);
    target.step(None).unwrap();
    assert_eq!(target.advance().unwrap(), Some(StopReason::Step));
    let registers = Registers::new(&target.core().processor);
    assert_eq!(single(&registers, 1), Some(1));
    assert_eq!(single(&registers, 32), Some(BASE + 4));
    assert_eq!(registers.pc(), BASE + 4);
    target.resume(None).unwrap();
    assert_eq!(target.advance().unwrap(), None);
    assert_eq!(
        target.advance().unwrap(),
        Some(StopReason::Breakpoint(BASE, BreakpointType::Software))
    );
    assert!(matches!(target.remove_sw_breakpoint(BASE, 4), Ok(true)));
    target.resume(None).unwrap();
    assert_eq!(target.advance().unwrap(), None);
    assert_eq!(target.advance().unwrap(), None);
    assert_eq!(target.advance().unwrap(), Some(StopReason::RunLimit));
    let mut after = [0; 8];
    target.inspect(BASE, &mut after).unwrap();
    assert_eq!(before, after);
    assert!(target.inspect(u64::MAX, &mut [0; 2]).is_err());
    assert!(target.inspect(BASE, &mut [0; 1025]).is_err());
    assert!(target.write_addrs(BASE, &[0; 4]).is_err());
    assert!(target.write_registers(&registers).is_err());
    assert!(target.add_hw_breakpoint(BASE + 1, 2).is_err());
    assert!(matches!(target.add_hw_breakpoint(BASE + 2, 2), Ok(true)));
    assert_eq!(
        target.add_sw_breakpoint(0x4000_0000, 4).is_ok(),
        P::SUPERVISOR
    );
}

fn enter<P: PrivilegeModel, M: GdbModel<Rv64>>(
    target: &mut Wide<P, M>,
    previous: u64,
    address: u64,
) {
    let processor = &mut target.core.processor;
    for (register, value) in [(0x300, previous << 11), (0x341, address)] {
        processor
            .privilege
            .write_csr(&mut processor.machine_registers, register, value)
            .unwrap();
    }
    target.step(None).unwrap();
    assert_eq!(target.advance().unwrap(), Some(StopReason::Step));
}

#[test]
fn every_64_bit_machine_breaks_steps_and_reads_registers_and_memory() {
    breaks_steps_and_reads::<MachineOnly, Flat>(PLAIN);
    breaks_steps_and_reads::<Supervisor, Flat>(FLOAT);
    breaks_steps_and_reads::<MachineOnly, Rvy>(CAPABILITY);
    breaks_steps_and_reads::<Supervisor, Rvy>(BOTH);
}

#[test]
fn floating_point_registers_and_their_control_register_are_read() {
    let mut hart = target::<Supervisor, Flat>(FLOAT, &LOOP).into_core();
    hart.processor.float_registers[1] = 1.5_f64.to_bits();
    hart.processor.float_control = 0x61;
    let registers = Registers::new(&hart.processor);
    assert_eq!(single(&registers, 34), Some(1.5_f64.to_bits()));
    assert_eq!(single(&registers, 66), Some(1));
    assert_eq!(single(&registers, 67), Some(3));
    assert_eq!(single(&registers, 68), Some(0x61));
    assert_eq!(single(&registers, 4161), Some(3));
}

fn listed(description: &str) -> Vec<(String, usize, usize)> {
    description
        .lines()
        .filter(|line| line.contains("<reg "))
        .map(|line| {
            let field = |name: &str| {
                let start =
                    line.find(&format!("{name}=\"")).unwrap() + name.len() + 2;
                line[start..].split('"').next().unwrap().to_owned()
            };
            (
                field("name"),
                field("bitsize").parse().unwrap(),
                field("regnum").parse().unwrap(),
            )
        })
        .collect()
}

#[test]
fn machine_registers_are_read_while_the_guest_is_in_user_mode() {
    let mut target = target::<Supervisor, Flat>(FLOAT, &[MRET]);
    enter(&mut target, 0, BASE + 0x100);
    let registers = Registers::new(&target.core().processor);
    assert_eq!(single(&registers, 4161), Some(0));
    assert_eq!(single(&registers, 32), Some(BASE + 0x100));
    assert!(single(&registers, 65 + 0x300).is_some());
    assert_eq!(single(&registers, 65 + 0x341), Some(BASE + 0x100));
    assert_eq!(single(&registers, 65 + 0x180), Some(0));
    assert!(target.monitor(b"status").unwrap().contains("mode=User"));
}

#[test]
fn memory_is_read_through_the_guests_page_table() {
    let mapped = 0x10_0000_0000_u64;
    let mut target = target::<Supervisor, Flat>(FLOAT, &[MRET]);
    let entry = (BASE >> 12) << 10 | 0xcb;
    let hart = &mut target.core;
    load_raw(
        &mut hart.bus,
        BASE + 0x1000 + (mapped >> 30) * 8,
        &entry.to_le_bytes(),
    )
    .unwrap();
    let processor = &mut hart.processor;
    processor
        .privilege
        .write_csr(
            &mut processor.machine_registers,
            0x180,
            8 << 60 | (BASE + 0x1000) >> 12,
        )
        .unwrap();
    let mut direct = [0; 4];
    target.inspect(BASE, &mut direct).unwrap();
    assert_eq!(direct, MRET.to_le_bytes());
    assert!(target.inspect(mapped, &mut [0; 4]).is_err());
    enter(&mut target, 1, mapped + 0x100);
    let mut through = [0; 4];
    target.inspect(mapped, &mut through).unwrap();
    assert_eq!(through, MRET.to_le_bytes());
    target.inspect(mapped + 0x1ff8, &mut [0; 16]).unwrap();
    assert!(target.inspect(BASE, &mut [0; 4]).is_err());
    assert!(target.inspect(mapped + (1 << 30), &mut [0; 4]).is_err());
    assert!(target.inspect(1 << 38, &mut [0; 4]).is_err());
    assert!(
        target
            .monitor(b"status")
            .unwrap()
            .contains("mode=Supervisor")
    );
}

fn packet_matches<P: PrivilegeModel, M: GdbModel<Rv64>>(
    extensions: &str,
) -> String {
    let mut hart = target::<P, M>(extensions, &LOOP).into_core();
    hart.processor.float_registers[1] = 1.5_f64.to_bits();
    hart.processor.float_control = 0x61;
    let registers = Registers::new(&hart.processor);
    let description = registers.description();
    let mut bytes = Vec::new();
    registers.gdb_serialize(|byte| bytes.push(byte));
    let mut offset = 0;
    for (name, bits, number) in listed(&description) {
        let width = bits / 8;
        let packet: Option<Vec<u8>> =
            bytes[offset..offset + width].iter().copied().collect();
        let packet = packet.map(|bytes| {
            let mut whole = [0; 8];
            whole[..width].copy_from_slice(&bytes);
            u64::from_le_bytes(whole)
        });
        assert_eq!(single(&registers, number), packet, "{name}");
        offset += width;
    }
    assert_eq!(offset, bytes.len());
    assert!(description.contains("<architecture>riscv:rv64</architecture>"));
    assert!(description.contains("name=\"x31\" bitsize=\"64\" regnum=\"31\""));
    assert!(description.contains("name=\"pc\" bitsize=\"64\" regnum=\"32\""));
    assert!(
        description.contains("name=\"misa\" bitsize=\"64\" regnum=\"834\"")
    );
    assert!(!description.contains("mstatush"));
    assert_eq!(
        single(&registers, 65 + 0x301),
        Some(hart.processor.machine_registers.isa)
    );
    assert!(RegisterId::from_raw_id(4162).is_none());
    description
}

fn shows_capabilities<P: PrivilegeModel>(extensions: &str) {
    let mut hart =
        target::<P, Rvy>(extensions, &[0xffff_ffff, 0x0000_0013]).into_core();
    let held = Capability128::INFINITE.with_address(BASE + 0x100);
    hart.processor.set_capability(1, held.with_bounds(64, true));
    hart.processor.set_capability(2, Capability128::integer(7));
    hart.bus.set_tag(BASE + 0x20);
    let mut target = Target::new(hart, Some(4));
    let help = target.monitor(b"help").unwrap();
    assert!(help.contains("capability x0..x31|pcc"), "{help}");
    let program_counter = target.monitor(b"capability pcc").unwrap();
    assert!(
        program_counter.starts_with(
            "pcc: metadata=0xf007e00000000000 0x0000000080000000 (tag=1 "
        ),
        "{program_counter}"
    );
    assert_eq!(
        target.monitor(b"capability x1").unwrap(),
        format!(
            "x1: metadata=0x{:016x} 0x0000000080000100 (tag=1 \
             base=0x0000000080000100 top=0x80000140 perms={:#07x})\n",
            held.with_bounds(64, true).metadata,
            held.permissions() | permissions::LEVELS
        )
    );
    let integer = target.monitor(b"capability x2").unwrap();
    assert!(integer.contains("0x0000000000000007 (tag=0 "), "{integer}");
    assert!(target.monitor(b"capability x0").unwrap().contains("tag=0"));
    assert!(
        target
            .monitor(b"capability mtvec")
            .unwrap()
            .contains("tag=1")
    );
    assert!(
        target
            .monitor(b"capability mtidc")
            .unwrap()
            .contains("tag=0")
    );
    assert_eq!(target.monitor(b"capability stvec").is_ok(), P::SUPERVISOR);
    assert_eq!(target.monitor(b"capability stidc").is_ok(), P::SUPERVISOR);
    for unknown in ["x32", "c1", "mtcc", ""] {
        assert_eq!(
            target
                .monitor(format!("capability {unknown}").as_bytes())
                .unwrap_err(),
            if unknown.is_empty() {
                "unknown monitor command; use monitor help"
            } else {
                "unknown capability register"
            }
        );
    }
    assert_eq!(
        target.monitor(b"tags 0x80000010 3").unwrap(),
        "stored tags, at physical addresses:\n0x0000000080000010: \
         0\n0x0000000080000020: 1\n0x0000000080000030: 0\n"
    );
    assert!(target.monitor(b"tags 0x80000008 1").is_err());
    assert!(target.monitor(b"tags 0x80000000 65").is_err());
    assert!(target.monitor(b"tags 0x90000000 1").is_err());
    target.step(None).unwrap();
    assert_eq!(target.advance().unwrap(), Some(StopReason::Fault(2)));
    let fault = target.monitor(b"fault").unwrap();
    assert!(
        fault.contains(
            "value=0x00000000ffffffff pre_pcc=0xf007e00000000000/1 \
             resulting_pc="
        ),
        "{fault}"
    );
    let registers = Registers::new(&target.core().processor);
    assert_eq!(single(&registers, 1), Some(BASE + 0x100));
}

fn single(registers: &Registers, number: usize) -> Option<u64> {
    let (id, _) = RegisterId::from_raw_id(number)?;
    let mut bytes = [0; 8];
    let size = registers.register(id, &mut bytes)?;
    (size != 0).then(|| u64::from_le_bytes(bytes))
}

fn target<P: PrivilegeModel, M: GdbModel<Rv64>>(
    extensions: &str,
    words: &[u32],
) -> Wide<P, M> {
    let mut configuration = crate::profile::reader::read(&format!(
        "name='debugger-test'\n[emulator]\nprocessor='rv64'\n\
         extensions=[{extensions}]\nmemory={{ram={{start=0x80000000,\
         length=0x10000}}}}\n"
    ))
    .unwrap();
    configuration.timing = TimingPolicy::PerStep;
    configuration.steps_per_tick = 1;
    let mut hart = configuration
        .build_wide_machine::<P, M>(false, BASE, &firmware_from_words(words))
        .unwrap();
    hart.trap_policy = TrapPolicy::Vector;
    Target::new(hart, Some(4))
}

#[test]
fn the_monitor_shows_whole_capabilities_and_stored_tags() {
    shows_capabilities::<MachineOnly>(CAPABILITY);
    shows_capabilities::<Supervisor>(BOTH);
}

#[test]
fn the_register_packet_the_single_reads_and_the_description_agree() {
    for description in [
        packet_matches::<MachineOnly, Flat>(PLAIN),
        packet_matches::<MachineOnly, Rvy>(CAPABILITY),
    ] {
        assert!(!description.contains("riscv.fpu"));
        assert!(!description.contains("satp"));
        assert!(!description.contains("priv"));
    }
    for description in [
        packet_matches::<Supervisor, Flat>(FLOAT),
        packet_matches::<Supervisor, Rvy>(BOTH),
    ] {
        assert!(description.contains(
            "name=\"f1\" bitsize=\"64\" regnum=\"34\" type=\"ieee_double\""
        ));
        assert!(
            description.contains("name=\"fcsr\" bitsize=\"32\" regnum=\"68\"")
        );
        assert!(
            description.contains("name=\"satp\" bitsize=\"64\" regnum=\"449\"")
        );
        assert!(
            description
                .contains("name=\"priv\" bitsize=\"64\" regnum=\"4161\"")
        );
    }
    let single_precision = "'i','m','a','f','c','zicsr','zifencei'";
    let description = packet_matches::<MachineOnly, Flat>(single_precision);
    assert!(description.contains(
        "name=\"f1\" bitsize=\"32\" regnum=\"34\" type=\"ieee_single\""
    ));
}

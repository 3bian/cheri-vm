// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod compressed_reference;
mod csr_reference;
mod reference;
mod reference_input;
mod revoker;

use crate::hart::Hart;
use crate::hart::cheriot_rv32e::{CheriotRv32e, Processor};
use crate::machine::bus::Bus;
use crate::machine::memory::{MemoryKind, MemoryRegion, Tagging};

#[test]
fn a_new_instruction_written_over_a_cached_one_runs_next() {
    const START: u32 = 0x8000_0000;
    let mut bus = Bus::new();
    bus.add_memory(
        MemoryRegion::new(
            "ram".into(),
            START.into(),
            0x1000,
            MemoryKind::Ram,
            Tagging::Cheriot,
        )
        .unwrap(),
    )
    .unwrap();
    let mut computer = Hart::<CheriotRv32e>::new(Processor::new(START), bus);
    computer.timing = crate::machine::TimingPolicy::PerStep;
    for (instruction, expected) in
        [(0x0010_0093_u32, 1), (0x0020_0093, 2), (0x0010_0093, 1)]
    {
        computer.bus.write(START.into(), 4, instruction).unwrap();
        computer.processor.program_counter = START;
        computer.step().unwrap();
        assert_eq!(
            computer.processor.read_register(1).unwrap().address(),
            expected
        );
    }
}

// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::decode::standard;
use crate::isa::extension::ExtensionSet;
use crate::isa::extension::privileged::{machine, supervisor};
use crate::isa::instruction::Instruction;

#[test]
fn a_machine_without_supervisor_mode_does_not_decode_sret() {
    let plain = ExtensionSet::named(["i", "zicsr"]).unwrap();
    let modes = ExtensionSet::named(["i", "zicsr", "s", "u"]).unwrap();
    assert!(standard(plain, 64, 0x1020_0073).is_none());
    assert_eq!(
        standard(modes, 64, 0x1020_0073).unwrap().instruction,
        Instruction::SupervisorReturn
    );
    assert_eq!(
        standard(plain, 64, 0x3020_0073).unwrap().instruction,
        Instruction::MachineReturn
    );
}

#[test]
fn the_returns_the_wait_and_the_fence_decode_from_their_own_encodings() {
    assert_eq!(machine(0x3020_0073), Some(Instruction::MachineReturn));
    assert_eq!(machine(0x1050_0073), Some(Instruction::WaitForInterrupt));
    assert_eq!(machine(0x1020_0073), None);
    assert_eq!(supervisor(0x1020_0073), Some(Instruction::SupervisorReturn));
    assert_eq!(supervisor(0x12b5_0073), Some(Instruction::SupervisorFence));
    assert_eq!(supervisor(0x12b5_00f3), None);
    assert_eq!(supervisor(0x3020_0073), None);
}

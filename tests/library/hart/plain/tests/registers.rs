// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::hart::plain::Processor;
use crate::isa::extension::ExtensionSet;
use crate::isa::xlen::Rv32;

#[test]
fn public_register_access_rejects_all_unavailable_indices_without_mutation() {
    for base in [ExtensionSet::I, ExtensionSet::E] {
        let profile = base
            .with(ExtensionSet::M)
            .with(ExtensionSet::C)
            .with(ExtensionSet::ZICSR);
        let mut processor = Processor::<Rv32>::new(profile, 0);
        for index in 0..profile.register_count() as u8 {
            processor
                .write_register(index, u32::from(index) + 5)
                .unwrap();
        }
        assert_eq!(processor.read_register(0).unwrap(), 0);
        let before = processor.registers().to_vec();
        for index in profile.register_count() as u8..=u8::MAX {
            assert!(processor.read_register(index).is_err());
            assert!(processor.write_register(index, u32::MAX).is_err());
            assert_eq!(processor.registers(), before);
        }
        for index in 1..profile.register_count() as u8 {
            assert_eq!(
                processor.read_register(index).unwrap(),
                u32::from(index) + 5
            );
        }
    }
}

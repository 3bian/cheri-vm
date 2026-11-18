// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::extension::*;

#[test]
fn a_set_is_built_from_names_and_gives_them_back_in_the_registry_order() {
    let set = ExtensionSet::named(["zicsr", "c", "i"]).unwrap();
    assert!(set.contains(ExtensionSet::I.with(ExtensionSet::C)));
    assert!(!set.contains(ExtensionSet::M));
    assert_eq!(set.names().collect::<Vec<_>>(), ["i", "c", "zicsr"]);
    assert_eq!(ExtensionSet::named(["i", "q"]), Err("q"));
    assert_eq!(ExtensionSet::named(["i", "m", "i"]), Err("i"));
}

#[test]
fn every_extension_has_one_name_and_one_bit() {
    for (index, extension) in EXTENSIONS.iter().enumerate() {
        assert_eq!(named(extension.name).unwrap().set, extension.set);
        assert!(
            extension.standard.is_some()
                || extension.compressed.is_some()
                || ["u", "zylg", "svyrg"].contains(&extension.name)
        );
        assert!(!extension.summary.is_empty());
        for other in &EXTENSIONS[index + 1..] {
            assert_ne!(extension.name, other.name);
            assert!(!extension.set.contains(other.set));
        }
    }
    assert!(named("x").is_none());
}

#[test]
fn the_set_gives_the_register_count_and_the_instruction_alignment() {
    assert_eq!(ExtensionSet::I.register_count(), 32);
    assert_eq!(ExtensionSet::E.register_count(), 16);
    assert_eq!(ExtensionSet::I.instruction_alignment(), 4);
    assert_eq!(
        ExtensionSet::I
            .with(ExtensionSet::C)
            .instruction_alignment(),
        2
    );
}

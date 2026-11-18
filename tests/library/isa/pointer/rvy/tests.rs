// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::pointer::rvy::{Capability128, Window, permissions};
use crate::isa::pointer::{Access, PointerModel, Rvy, Violation};

const WHOLE: u128 = 1 << 64;

#[test]
fn a_capability_with_reserved_bits_or_bad_bounds_is_unsound_and_grants_nothing()
{
    for metadata in [
        Capability128::INFINITE.metadata | 1 << 44,
        Capability128::INFINITE.metadata | 1 << 53,
        1 << 45,
        1 << 49,
        (1 << 50) | (1 << 47),
        Capability128::INFINITE.metadata | 1 << 3,
        Capability128::INFINITE.metadata | 7 << 14 | 7,
    ] {
        let capability = Capability128 {
            address: 0x1000,
            metadata,
            tag: true,
        };
        assert!(!capability.is_sound(), "{metadata:#x}");
        assert_eq!(capability.bounds(), (0, 0));
        assert_eq!(capability.permissions(), permissions::RESERVED_ONES);
        assert!(!capability.with_address(0x1000).tag);
        assert!(!capability.without_permissions(0).tag);
        assert_eq!(
            Rvy::check_access(capability, 0x1000, 1, Access::Load),
            Err(Violation::Permission)
        );
    }
}

#[test]
fn a_fetch_window_admits_what_decoding_the_capability_each_time_would() {
    let mut state = 0x9e37_79b9_7f4a_7c15;
    let mut admitted = 0;
    for round in 0..20_000 {
        let base = random(&mut state) >> (random(&mut state) % 64);
        let length = random(&mut state) >> (random(&mut state) % 64);
        let mut capability = root(base).with_bounds(length, false);
        match round % 7 {
            0 => capability.tag = false,
            1 => capability = capability.with_seal(true),
            2 => {
                capability =
                    capability.without_permissions(permissions::EXECUTE)
            }
            3 => capability.metadata |= 1 << 30,
            _ => {}
        }
        let window = Window::of(capability);
        let (low, top) = capability.bounds();
        let edge = top.min(WHOLE - 1) as u64;
        for address in [
            low,
            low.wrapping_sub(1),
            low.wrapping_add(1),
            edge.wrapping_sub(3),
            edge.wrapping_sub(2),
            edge.wrapping_sub(1),
            edge,
            random(&mut state),
            0,
            u64::MAX,
            u64::MAX - 1,
        ] {
            let expected = fetch_by_decoding(capability, address);
            assert_eq!(
                window.admits(address),
                expected,
                "{capability:?} {address:#x}"
            );
            assert_eq!(window.covers(address), expected.is_ok());
            admitted += u32::from(expected.is_ok());
        }
    }
    assert!(admitted > 20_000, "{admitted}");
    let whole = Window::of(Capability128::INFINITE);
    assert_eq!(whole.admits(u64::MAX - 1), Ok(()));
    assert_eq!(whole.admits(u64::MAX), Err(Violation::Bounds));
    let one_byte = Window::of(root(0x2000).with_bounds(1, true));
    assert_eq!(one_byte.admits(0x2000), Err(Violation::Bounds));
}

#[test]
fn a_subset_has_no_more_permissions_and_no_wider_bounds() {
    let outer = root(0x1000).with_bounds(0x100, true);
    let inner = outer.with_address(0x1010).with_bounds(0x10, true);
    assert!(inner.is_subset_of(outer));
    assert!(!outer.is_subset_of(inner));
    assert!(outer.is_subset_of(outer));
    let weaker = outer.without_permissions(permissions::WRITE);
    assert!(weaker.is_subset_of(outer));
    assert!(!outer.is_subset_of(weaker));
}

#[test]
fn an_access_needs_a_tag_no_seal_the_permission_and_every_byte_in_bounds() {
    let data = root(0x2000)
        .with_bounds(16, true)
        .without_permissions(permissions::EXECUTE);
    let check = |capability, address, width, access| {
        Rvy::check_access(capability, address, width, access)
    };
    assert_eq!(check(data, 0x2000, 16, Access::Load), Ok(()));
    assert_eq!(check(data, 0x2008, 8, Access::Store), Ok(()));
    assert_eq!(check(data, 0x2009, 8, Access::Load), Err(Violation::Bounds));
    assert_eq!(check(data, 0x1fff, 1, Access::Load), Err(Violation::Bounds));
    let untagged = Capability128 { tag: false, ..data };
    assert_eq!(
        check(untagged, 0x2000, 1, Access::Load),
        Err(Violation::Tag)
    );
    assert_eq!(
        check(data.with_seal(true), 0x2000, 1, Access::Load),
        Err(Violation::Seal)
    );
    let read_only = data.without_permissions(permissions::WRITE);
    assert_eq!(check(read_only, 0x2000, 1, Access::Load), Ok(()));
    assert_eq!(
        check(read_only, 0x2000, 1, Access::Store),
        Err(Violation::Permission)
    );
    let fetch =
        |capability, address| Rvy::check_fetch(Window::of(capability), address);
    assert_eq!(fetch(data, 0x2000), Err(Violation::Permission));
    assert_eq!(fetch(root(0x2000), 0x2000), Ok(()));
    let code = root(0x2000).with_bounds(4, true);
    assert_eq!(fetch(code, 0x2002), Ok(()));
    assert_eq!(fetch(code, 0x2003), Err(Violation::Bounds));
}

#[test]
fn an_address_far_outside_the_bounds_cannot_be_represented() {
    let capability = root(0x4000_0000).with_bounds(0x10_0000, true);
    assert!(capability.tag);
    assert!(capability.with_address(0x4010_0000).tag);
    assert!(capability.with_address(0x3fff_f000).tag);
    assert!(!capability.with_address(0x8000_0000).tag);
    assert!(!capability.with_address(0).tag);
    assert_eq!(capability.with_address(0x8000_0000).address, 0x8000_0000);
    assert!(root(0).with_address(u64::MAX).tag);
}

#[test]
fn an_address_inside_the_bounds_decodes_to_the_same_bounds() {
    let mut state = 0x2545_f491_4f6c_dd1d;
    let mut moved = 0;
    for _ in 0..20_000 {
        let base = random(&mut state) >> (random(&mut state) % 64);
        let length = random(&mut state) >> (random(&mut state) % 64);
        let capability = root(base).with_bounds(length, false);
        let (low, top) = capability.bounds();
        let span = top - u128::from(low);
        if span == 0 {
            continue;
        }
        for offset in
            [0, span - 1, span / 2, u128::from(random(&mut state)) % span]
        {
            let address = low.wrapping_add(offset as u64);
            let there = Capability128 {
                address,
                ..capability
            };
            assert_eq!(
                there.bounds(),
                (low, top),
                "{capability:?} {address:#x}"
            );
            assert_eq!(capability.with_address(address), there);
            assert_eq!(there.reach(), (there.permissions(), low, top));
            moved += 1;
        }
    }
    assert!(moved > 70_000, "{moved}");
}

#[test]
fn bounds_can_only_shrink_and_a_sealed_capability_cannot_change() {
    let outer = root(0x1000).with_bounds(0x100, true);
    assert!(outer.with_address(0x1080).with_bounds(0x80, true).tag);
    assert!(!outer.with_address(0x1080).with_bounds(0x81, true).tag);
    assert!(!outer.with_address(0xff0).with_bounds(0x20, true).tag);

    let sealed = outer.with_seal(true);
    assert!(sealed.sealed() && sealed.tag);
    assert!(!sealed.with_address(0x1000).tag);
    assert!(!sealed.with_bounds(0x10, true).tag);
    assert!(!sealed.without_permissions(permissions::WRITE).tag);
    assert!(sealed.without_permissions(0).tag);
    assert_eq!(sealed.with_seal(false), outer);
}

#[test]
fn clearing_a_permission_clears_the_ones_that_depend_on_it() {
    let all = Capability128::INFINITE;
    let held =
        |capability: Capability128| capability.permissions() & permissions::ALL;
    assert_eq!(
        held(all.without_permissions(permissions::EXECUTE)),
        permissions::ALL & !(permissions::EXECUTE | permissions::SYSTEM)
    );
    assert_eq!(
        held(all.without_permissions(permissions::READ)),
        permissions::ALL & !(permissions::READ | permissions::LOAD_MUTABLE)
    );
    assert_eq!(
        held(all.without_permissions(permissions::READ | permissions::WRITE)),
        permissions::EXECUTE | permissions::SYSTEM | permissions::SOFTWARE
    );
    assert_eq!(
        held(all.without_permissions(permissions::CAPABILITY)),
        permissions::ALL
            & !(permissions::CAPABILITY | permissions::LOAD_MUTABLE)
    );
    assert_eq!(
        held(all.without_permissions(0x80)),
        permissions::ALL & !0x80
    );
    assert_eq!(held(all.without_permissions(u64::MAX)), 0);
    assert!(all.without_permissions(u64::MAX).tag);
}

#[test]
fn every_requested_range_is_covered_and_an_exact_one_is_met_exactly() {
    let mut state = 0x9e37_79b9_7f4a_7c15;
    for _ in 0..200_000 {
        let shift = random(&mut state) % 64;
        let length = random(&mut state) >> shift;
        let base = (random(&mut state) >> (random(&mut state) % 64))
            .min(u64::MAX - length);
        let top = u128::from(base) + u128::from(length);
        let rounded = root(base).with_bounds(length, false);
        let (found_base, found_top) = rounded.bounds();
        assert!(rounded.tag, "{base:#x} {length:#x}");
        assert!(
            found_base <= base && found_top >= top,
            "{base:#x} {length:#x}"
        );
        assert!(found_top <= WHOLE);
        let slack = (found_top - u128::from(found_base)) - u128::from(length);
        assert!(
            slack <= u128::from(length) / 256 + 16,
            "{base:#x} {length:#x}"
        );

        let exact = root(base).with_bounds(length, true);
        assert_eq!(exact.tag, (found_base, found_top) == (base, top));

        for address in [base, base + length / 2, base + length] {
            let moved = rounded.with_address(address);
            assert!(moved.tag, "{base:#x} {length:#x} {address:#x}");
            assert_eq!(moved.bounds(), (found_base, found_top));
        }
        let again = rounded.with_bounds(length, false);
        assert!(again.tag);
        assert_eq!(again.bounds(), (found_base, found_top));
    }
}

fn fetch_by_decoding(
    capability: Capability128,
    address: u64,
) -> Result<(), Violation> {
    if !capability.tag {
        Err(Violation::Tag)
    } else if capability.sealed() {
        Err(Violation::Seal)
    } else if capability.permissions() & permissions::EXECUTE == 0 {
        Err(Violation::Permission)
    } else if !capability.contains(address, 2) {
        Err(Violation::Bounds)
    } else {
        Ok(())
    }
}

#[test]
fn levels_add_a_global_flag_and_two_permissions_that_depend_on_others() {
    use permissions::{
        ALL, CAPABILITY, GLOBAL, LEVELS, LOAD_GLOBAL, READ, RESERVED_ONES,
        STORE_LOCAL, WRITE,
    };
    let root = Capability128::infinite(true);
    assert_eq!(root.metadata, 0xf01f_e800_0000_0000);
    assert!(root.tag && root.is_sound() && root.has_infinite_bounds());
    assert_eq!(root.permissions(), ALL | LEVELS | RESERVED_ONES);
    assert_eq!(Capability128::infinite(false), Capability128::INFINITE);
    assert_eq!(Capability128::INFINITE.permissions() & LEVELS, 0);
    assert_eq!(RESERVED_ONES & LEVELS, 0);

    let levels = |capability: Capability128| capability.permissions() & LEVELS;
    assert_eq!(levels(root.without_permissions(READ)), GLOBAL | STORE_LOCAL);
    assert_eq!(
        levels(root.without_permissions(WRITE)),
        GLOBAL | LOAD_GLOBAL
    );
    assert_eq!(levels(root.without_permissions(CAPABILITY)), GLOBAL);
    assert_eq!(
        levels(root.without_permissions(STORE_LOCAL)),
        GLOBAL | LOAD_GLOBAL
    );
    let local = root.without_permissions(GLOBAL);
    assert!(local.tag);
    assert_eq!(levels(local), LOAD_GLOBAL | STORE_LOCAL);
    assert!(local.is_subset_of(root));
    assert!(!root.is_subset_of(local));

    let sealed = root.with_seal(true);
    let made_local = sealed.without_permissions(GLOBAL);
    assert!(made_local.tag && made_local.sealed());
    assert_eq!(levels(made_local), LOAD_GLOBAL | STORE_LOCAL);
    assert!(!sealed.without_permissions(LOAD_GLOBAL).tag);
    assert!(!sealed.without_permissions(GLOBAL | WRITE).tag);

    for (kept, sound) in [
        (0b0000_0111, true),
        (0b0100_0101, true),
        (0b1000_0011, true),
        (0b1100_0111, true),
        (0b0100_0011, false),
        (0b0100_0100, false),
        (0b1000_0101, false),
        (0b1000_0010, false),
    ] {
        let capability = Capability128 {
            metadata: kept << 45,
            ..root
        };
        assert_eq!(capability.is_sound(), sound, "{kept:#010b}");
    }
    let flagged = Capability128 {
        metadata: 1 << 43,
        ..root
    };
    assert!(flagged.is_sound());
    assert_eq!(flagged.permissions(), GLOBAL | RESERVED_ONES);
}

#[test]
fn moving_an_address_keeps_the_tag_only_while_the_bounds_decode_the_same() {
    let mut state = 0x1234_5678_9abc_def1;
    let (mut kept, mut lost) = (0, 0);
    for _ in 0..20_000 {
        let base = random(&mut state) >> (random(&mut state) % 64);
        let length = random(&mut state) >> (random(&mut state) % 48);
        let capability = root(base).with_bounds(length, false);
        let distance = random(&mut state) >> (random(&mut state) % 64);
        let address = base.wrapping_add(distance);
        let there = Capability128 {
            address,
            ..capability
        };
        let same = there.bounds() == capability.bounds();
        assert!(!capability.tag || capability.is_sound(), "{capability:?}");
        assert_eq!(
            capability.with_address(address).tag,
            capability.tag && same,
            "{capability:?} {address:#x}"
        );
        kept += u32::from(same);
        lost += u32::from(!same);
    }
    assert!(kept > 1_000 && lost > 1_000, "{kept} {lost}");
    let past_the_end =
        root(0xb8c2_0000_0000_0000).with_bounds(0x7fff_ffff_ffff_ffff, false);
    assert!(!past_the_end.tag);
    let widest = root(1).with_bounds(u64::MAX - 1, false);
    assert!(widest.tag && widest.is_sound());
    assert_eq!(widest.bounds(), (0, WHOLE));
    assert!(!root(1).with_bounds(u64::MAX - 1, true).tag);
    let narrow = root(0x4000).with_bounds(0x1000, true);
    assert!(!narrow.with_bounds(u64::MAX - 0x4000, false).tag);
    let unsound = Capability128 {
        metadata: Capability128::INFINITE.metadata | 1 << 30,
        ..root(0x1000)
    };
    assert_eq!(unsound.reach(), (permissions::RESERVED_ONES, 0, 0));
    assert!(!unsound.with_address(0x1000).tag);
}

fn random(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn root(address: u64) -> Capability128 {
    Capability128 {
        address,
        ..Capability128::INFINITE
    }
}

#[test]
fn small_bounds_are_exact_to_the_byte_and_large_ones_are_rounded_outward() {
    let small = root(0x8000_1003).with_bounds(100, true);
    assert!(small.tag);
    assert_eq!(small.bounds(), (0x8000_1003, 0x8000_1003 + 100));
    assert!(!small.has_infinite_bounds());

    let largest_exact = root(0x1001).with_bounds(4095, true);
    assert_eq!(largest_exact.bounds(), (0x1001, 0x1001 + 4095));
    assert!(largest_exact.tag);

    let page = root(0x7000).with_bounds(4096, true);
    assert!(page.tag);
    assert_eq!(page.bounds(), (0x7000, 0x8000));

    let odd = root(0x7001).with_bounds(4096, true);
    assert!(!odd.tag);
    let rounded = root(0x7001).with_bounds(4096, false);
    assert!(rounded.tag);
    assert_eq!(rounded.bounds(), (0x7000, 0x8008));

    let everything = root(0).with_bounds(u64::MAX, false);
    assert!(everything.tag);
    assert_eq!(everything.bounds(), (0, WHOLE));
}

#[test]
fn the_alignment_mask_makes_a_length_that_can_be_met_exactly() {
    assert_eq!(Capability128::alignment_mask(4095), u64::MAX);
    assert_eq!(Capability128::alignment_mask(4096), !7);
    assert_eq!(Capability128::alignment_mask(1 << 20), !0x7ff);
    let mut state = 0x1234_5678_9abc_def1;
    for _ in 0..100_000 {
        let length = random(&mut state) >> (random(&mut state) % 63 + 1);
        let mask = Capability128::alignment_mask(length);
        let rounded = length.wrapping_add(!mask) & mask;
        let base = random(&mut state) & mask & (u64::MAX >> 2);
        let capability = root(base).with_bounds(rounded, true);
        assert!(capability.tag, "{length:#x} {base:#x}");
        assert_eq!(
            capability.bounds(),
            (base, u128::from(base) + u128::from(rounded))
        );
    }
}

#[test]
fn the_null_and_infinite_capabilities_cover_everything_and_only_one_grants_anything()
 {
    let null = Capability128::default();
    assert_eq!(null.bounds(), (0, WHOLE));
    assert_eq!(null.permissions(), permissions::RESERVED_ONES);
    assert!(null.is_sound() && !null.tag && !null.sealed());

    let infinite = Capability128::INFINITE;
    assert_eq!(infinite.bounds(), (0, WHOLE));
    assert_eq!(root(u64::MAX).bounds(), (0, WHOLE));
    assert!(infinite.has_infinite_bounds());
    assert_eq!(
        infinite.permissions(),
        permissions::ALL | permissions::RESERVED_ONES
    );
    assert_eq!(infinite.metadata, 0xf007_e000_0000_0000);
}

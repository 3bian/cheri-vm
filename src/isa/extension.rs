// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod decoding;
mod extension_set;

pub use decoding::Decoding;
pub use extension_set::ExtensionSet;

pub mod a;

pub mod base;

pub mod c;

pub mod cheriot;

pub mod d;

pub mod f;

mod fields;

pub mod float;

pub mod m;

pub mod privileged;

pub mod rvy;

pub mod zba;

pub mod zbb;

pub mod zbc;

pub mod zbs;

pub mod zcb;

pub mod zicond;

pub mod zicsr;

pub mod zifencei;

pub mod zkn;

pub const EXTENSIONS: [Extension; 22] = [
    Extension {
        compressed: Some(cheriot::compressed),
        name: "cheriot",
        set: ExtensionSet::CHERIOT,
        standard: Some(cheriot::standard),
        summary: "The CHERIoT capability instructions",
    },
    Extension {
        compressed: Some(rvy::base::compressed),
        name: "y",
        set: ExtensionSet::Y,
        standard: Some(|word, xlen| rvy::base::standard(word, xlen).into()),
        summary: "The RVY capability instructions, at draft v0.9.10.1 in the \
                  RV64LYA format",
    },
    Extension {
        compressed: None,
        name: "zylg",
        set: ExtensionSet::ZYLG,
        standard: None,
        summary: "Local and global RVY capabilities, with the store-local and \
                  load-global permissions",
    },
    Extension {
        compressed: None,
        name: "i",
        set: ExtensionSet::I,
        standard: Some(|word, xlen| base::standard(word, xlen).into()),
        summary: "The base integer instructions, with 32 registers",
    },
    Extension {
        compressed: None,
        name: "e",
        set: ExtensionSet::E,
        standard: Some(|word, xlen| base::standard(word, xlen).into()),
        summary: "The base integer instructions, with 16 registers",
    },
    Extension {
        compressed: None,
        name: "m",
        set: ExtensionSet::M,
        standard: Some(|word, xlen| m::standard(word, xlen).into()),
        summary: "Multiply and divide",
    },
    Extension {
        compressed: None,
        name: "a",
        set: ExtensionSet::A,
        standard: Some(|word, xlen| a::standard(word, xlen).into()),
        summary: "Atomic memory operations, and load-reserved with \
                  store-conditional",
    },
    Extension {
        compressed: Some(|halfword, xlen| c::compressed(halfword, xlen).into()),
        name: "c",
        set: ExtensionSet::C,
        standard: None,
        summary: "The 16-bit forms of common instructions",
    },
    Extension {
        compressed: Some(|halfword, xlen| f::compressed(halfword, xlen).into()),
        name: "f",
        set: ExtensionSet::F,
        standard: Some(|word, xlen| f::standard(word, xlen).into()),
        summary: "Single-precision floating point",
    },
    Extension {
        compressed: Some(|halfword, xlen| d::compressed(halfword, xlen).into()),
        name: "d",
        set: ExtensionSet::D,
        standard: Some(|word, xlen| d::standard(word, xlen).into()),
        summary: "Double-precision floating point",
    },
    Extension {
        compressed: None,
        name: "zicsr",
        set: ExtensionSet::ZICSR,
        standard: Some(|word, xlen| zicsr::standard(word, xlen).into()),
        summary: "The control and status register instructions, with mret and \
                  wfi",
    },
    Extension {
        compressed: None,
        name: "zifencei",
        set: ExtensionSet::ZIFENCEI,
        standard: Some(|word, xlen| zifencei::standard(word, xlen).into()),
        summary: "The instruction-fetch fence",
    },
    Extension {
        compressed: None,
        name: "zicond",
        set: ExtensionSet::ZICOND,
        standard: Some(|word, xlen| zicond::standard(word, xlen).into()),
        summary: "The conditional-zero instructions",
    },
    Extension {
        compressed: None,
        name: "zba",
        set: ExtensionSet::ZBA,
        standard: Some(|word, xlen| zba::standard(word, xlen).into()),
        summary: "Shift-and-add, for addresses",
    },
    Extension {
        compressed: None,
        name: "zbb",
        set: ExtensionSet::ZBB,
        standard: Some(|word, xlen| zbb::standard(word, xlen).into()),
        summary: "Bit counting, minimum and maximum, rotation and byte order",
    },
    Extension {
        compressed: None,
        name: "zbc",
        set: ExtensionSet::ZBC,
        standard: Some(|word, xlen| zbc::standard(word, xlen).into()),
        summary: "Carry-less multiplication",
    },
    Extension {
        compressed: None,
        name: "zbs",
        set: ExtensionSet::ZBS,
        standard: Some(|word, xlen| zbs::standard(word, xlen).into()),
        summary: "Single-bit set, clear, invert and extract",
    },
    Extension {
        compressed: Some(|halfword, xlen| {
            zcb::compressed(halfword, xlen).into()
        }),
        name: "zcb",
        set: ExtensionSet::ZCB,
        standard: None,
        summary: "The 16-bit forms of byte and halfword access, extension, \
                  not and multiply",
    },
    Extension {
        compressed: None,
        name: "zkn",
        set: ExtensionSet::ZKN,
        standard: Some(|word, xlen| zkn::standard(word, xlen).into()),
        summary: "The NIST cryptography instructions: AES, SHA-2, and the bit \
                  instructions they use",
    },
    Extension {
        compressed: None,
        name: "s",
        set: ExtensionSet::S,
        standard: Some(|word, _| privileged::supervisor(word).into()),
        summary: "Supervisor mode, with sret, sfence.vma and Sv39 translation",
    },
    Extension {
        compressed: None,
        name: "u",
        set: ExtensionSet::U,
        standard: None,
        summary: "User mode, with physical memory protection",
    },
    Extension {
        compressed: None,
        name: "svyrg",
        set: ExtensionSet::SVYRG,
        standard: None,
        summary: "Capability read generations and capability dirty tracking \
                  in Sv39 page-table entries",
    },
];

pub struct Extension {
    pub compressed: Option<fn(u16, u32) -> Decoding>,
    pub name: &'static str,
    pub set: ExtensionSet,
    pub standard: Option<fn(u32, u32) -> Decoding>,
    pub summary: &'static str,
}

pub fn named(name: &str) -> Option<&'static Extension> {
    EXTENSIONS.iter().find(|extension| extension.name == name)
}

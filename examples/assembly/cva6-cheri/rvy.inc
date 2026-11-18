/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* the RVY instructions as assembler macros, for an assembler that does */
/* not know them. the names are those of the RISC-V CHERI specification at */
/* draft v0.9.10.1; the encodings are the emulator's own reading of that */
/* draft, so a program built with this header shows the emulator agreeing */
/* with itself and is not a check of it. when an assembler knows the */
/* draft, this file goes and the sources stay. */

/* the 16-bit forms are left out, and turned off, because RVY gives some */
/* of them another meaning */
.option norvc

/* yadd rd, rs1, rs2: rs1 with its address moved on by rs2 */
.macro yadd rd, rs1, rs2
    .insn r 0x7b, 0, 0x03, \rd, \rs1, \rs2
.endm

/* yaddi rd, rs1, value: rs1 with its address moved on by a number */
.macro yaddi rd, rs1, value
    .insn i 0x7b, 4, \rd, \rs1, \value
.endm

/* ymv rd, rs1: a copy of rs1, tag and all, which `mv` does not give */
.macro ymv rd, rs1
    .insn r 0x7b, 0, 0x03, \rd, \rs1, x0
.endm

/* yaddrw rd, rs1, rs2: rs1 with its address set to rs2 */
.macro yaddrw rd, rs1, rs2
    .insn r 0x7b, 0, 0x0b, \rd, \rs1, \rs2
.endm

/* yhiw rd, rs1, rs2: the address of rs1 under the metadata in rs2, */
/* without a tag */
.macro yhiw rd, rs1, rs2
    .insn r 0x7b, 0, 0x01, \rd, \rs1, \rs2
.endm

/* ypermc rd, rs1, rs2: rs1 without the permissions whose bits rs2 has */
.macro ypermc rd, rs1, rs2
    .insn r 0x7b, 0, 0x13, \rd, \rs1, \rs2
.endm

/* ybndsw rd, rs1, rs2: rs1 with bounds of rs2 bytes from its address, */
/* and no tag if they cannot be held exactly */
.macro ybndsw rd, rs1, rs2
    .insn r 0x7b, 0, 0x1b, \rd, \rs1, \rs2
.endm

/* ybndsrw rd, rs1, rs2: the same, with the bounds rounded outward */
.macro ybndsrw rd, rs1, rs2
    .insn r 0x7b, 0, 0x23, \rd, \rs1, \rs2
.endm

/* ybndswi rd, rs1, length: bounds of 1 to 255 bytes. the instruction has */
/* longer lengths, in steps this macro does not work out */
.macro ybndswi rd, rs1, length
    .if (\length) < 1 || (\length) > 255
    .error "ybndswi takes a length from 1 to 255 here"
    .endif
    .insn i 0x7b, 5, \rd, \rs1, (\length) - 512
.endm

/* yseale rd, rs1: rs1 sealed as an entry, which a jump unseals */
.macro yseale rd, rs1
    .insn r 0x7b, 0, 0x17, \rd, x0, \rs1
.endm

/* yunseals rd, rs1, rs2: rs2 unsealed, by an rs1 that covers it */
.macro yunseals rd, rs1, rs2
    .insn r 0x7b, 0, 0x07, \rd, \rs1, \rs2
.endm

/* ybld rd, rs1, rs2: the bits of rs2 with a tag, if rs1 covers them */
.macro ybld rd, rs1, rs2
    .insn r 0x7b, 0, 0x0f, \rd, \rs1, \rs2
.endm

/* yeq rd, rs1, rs2: 1 if the two are the same, tag and all */
.macro yeq rd, rs1, rs2
    .insn r 0x7b, 0, 0x06, \rd, \rs1, \rs2
.endm

/* yss rd, rs1, rs2: 1 if rs2 is within the bounds and permissions of rs1 */
.macro yss rd, rs1, rs2
    .insn r 0x7b, 0, 0x0e, \rd, \rs1, \rs2
.endm

/* the readers: one field of rs1 as an integer */
.macro ybaser rd, rs1
    .insn r 0x7b, 0, 0x7a, \rd, \rs1, x0
.endm

.macro ypermr rd, rs1
    .insn r 0x7b, 0, 0x7a, \rd, \rs1, x1
.endm

.macro ytopr rd, rs1
    .insn r 0x7b, 0, 0x7a, \rd, \rs1, x2
.endm

.macro ylenr rd, rs1
    .insn r 0x7b, 0, 0x7a, \rd, \rs1, x3
.endm

.macro ytagr rd, rs1
    .insn r 0x7b, 0, 0x7a, \rd, \rs1, x4
.endm

.macro ytyper rd, rs1
    .insn r 0x7b, 0, 0x7a, \rd, \rs1, x5
.endm

.macro yhir rd, rs1
    .insn r 0x7b, 0, 0x7a, \rd, \rs1, x7
.endm

/* yamask rd, rs1: the mask that aligns an address for bounds of rs1 bytes */
.macro yamask rd, rs1
    .insn r 0x7b, 0, 0x78, \rd, \rs1, x0
.endm

/* ysh1add to ysh4add rd, rs1, rs2: rs2 with its address moved on by rs1 */
/* shifted left 1 to 4 places; the _uw forms take the low word of rs1 */
.macro ysh1add rd, rs1, rs2
    .insn r 0x7b, 0, 0x05, \rd, \rs1, \rs2
.endm

.macro ysh2add rd, rs1, rs2
    .insn r 0x7b, 0, 0x0d, \rd, \rs1, \rs2
.endm

.macro ysh3add rd, rs1, rs2
    .insn r 0x7b, 0, 0x15, \rd, \rs1, \rs2
.endm

.macro ysh4add rd, rs1, rs2
    .insn r 0x7b, 0, 0x1d, \rd, \rs1, \rs2
.endm

.macro ysh1add_uw rd, rs1, rs2
    .insn r 0x7b, 0, 0x25, \rd, \rs1, \rs2
.endm

.macro ysh2add_uw rd, rs1, rs2
    .insn r 0x7b, 0, 0x2d, \rd, \rs1, \rs2
.endm

.macro ysh3add_uw rd, rs1, rs2
    .insn r 0x7b, 0, 0x35, \rd, \rs1, \rs2
.endm

.macro ysh4add_uw rd, rs1, rs2
    .insn r 0x7b, 0, 0x3d, \rd, \rs1, \rs2
.endm

/* ly rd, offset, rs1: load the capability at rs1 plus the offset */
.macro ly rd, offset, rs1
    .insn i 0x7b, 1, \rd, \rs1, \offset
.endm

/* sy rs2, offset, rs1: store the capability rs2 there */
.macro sy rs2, offset, rs1
    .insn s 0x7b, 2, \rs2, \offset(\rs1)
.endm

/* lr_y, sc_y and amoswap_y: the atomic instructions on a capability */
.macro lr_y rd, rs1
    .insn r 0x7b, 3, 0x08, \rd, \rs1, x0
.endm

.macro sc_y rd, rs2, rs1
    .insn r 0x7b, 3, 0x0c, \rd, \rs1, \rs2
.endm

.macro amoswap_y rd, rs2, rs1
    .insn r 0x7b, 3, 0x04, \rd, \rs1, \rs2
.endm

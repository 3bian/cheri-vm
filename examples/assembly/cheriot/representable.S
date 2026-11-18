/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* crrl and cram on a length that needs rounding: the length that can be */
/* held, and the mask an address must be aligned by for it. run it with */
/* --trap-policy stop */

.include "common.inc"

.globl _start
_start:
    start
    li a1, 0x1234567
    crrl a5, a1
    show "crrl of 0x1234567: ", a5
    cram a5, a1
    show "cram of 0x1234567: ", a5
    li a1, 0x100
    crrl a5, a1
    show "crrl of 0x100:     ", a5
    cram a5, a1
    show "cram of 0x100:     ", a5
    cspecialr cs0, mtdc
    li a0, 0x80002345
    li a1, 0x1234567
    cram a2, a1
    and a0, a0, a2
    crrl a1, a1
    csetaddr cs0, cs0, a0
    csetboundsexact ca3, cs0, a1
    cgettag a5, ca3
    show "exact bounds after aligning by the mask, tag: ", a5
    ebreak

routines

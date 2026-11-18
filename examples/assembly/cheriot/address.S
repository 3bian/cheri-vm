/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* an address set, moved by a register and by an immediate, copied and */
/* subtracted; and a move too far from the bounds, which loses the tag. */
/* run it with --trap-policy stop */

.include "common.inc"

.globl _start
_start:
    start
    cspecialr cs0, mtdc
    li a0, 0x80001000
    csetaddr cs0, cs0, a0
    csetbounds cs0, cs0, 0x100
    cgetaddr a5, cs0
    show "csetaddr:                    ", a5
    li a1, 0x20
    cincoffset ca2, cs0, a1
    cgetaddr a5, ca2
    show "cincoffset by a register:    ", a5
    cincoffset ca2, ca2, -8
    cgetaddr a5, ca2
    show "cincoffset by an immediate:  ", a5
    cmove ca3, ca2
    csub a5, ca3, cs0
    show "the copy minus the first:    ", a5
    cgettag a5, ca3
    show "the copy's tag:              ", a5
    li a1, 0x100000
    cincoffset ca2, cs0, a1
    cgetaddr a5, ca2
    show "moved a megabyte away:       ", a5
    cgettag a5, ca2
    show "its tag:                     ", a5
    ebreak

routines

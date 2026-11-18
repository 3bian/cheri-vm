/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a mark set in the revocation bitmap: a capability to the marked memory */
/* that is loaded afterwards comes back without its tag. run it with */
/* --trap-policy stop */

.include "common.inc"

.globl _start
_start:
    start
    cspecialr cs0, mtdc
    li a0, 0x80008000
    csetaddr cs0, cs0, a0
    csetbounds cs0, cs0, 64
    cspecialr cs1, mtdc
    li a0, 0x80009000
    csetaddr cs1, cs1, a0
    csetbounds cs1, cs1, 16
    csc cs0, 0(cs1)
    clc ca1, 0(cs1)
    cgettag a5, ca1
    show "loaded before the mark, tag:      ", a5
    cspecialr ca2, mtdc
    li a0, 0x83000200
    csetaddr ca2, ca2, a0
    csetbounds ca2, ca2, 4
    li t0, 1
    sw t0, 0(ca2)
    clc ca1, 0(cs1)
    cgettag a5, ca1
    show "loaded after it, tag:             ", a5
    cgettag a5, cs0
    show "the copy already in a register:   ", a5
    sw zero, 0(ca2)
    clc ca1, 0(cs1)
    cgettag a5, ca1
    show "loaded once the mark is cleared:  ", a5
    ebreak

routines

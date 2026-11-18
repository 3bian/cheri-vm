/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a tag cleared on purpose, by csethigh, and by an integer written over */
/* a capability in memory. run it with --trap-policy stop */

.include "common.inc"

.globl _start
_start:
    start
    cspecialr cs0, mtdc
    li a0, 0x80002000
    csetaddr cs0, cs0, a0
    csetbounds cs0, cs0, 64
    cgettag a5, cs0
    show "a capability's tag:                 ", a5
    ccleartag ca2, cs0
    cgettag a5, ca2
    show "after ccleartag:                    ", a5
    cgethigh a1, cs0
    csethigh ca2, cs0, a1
    cgettag a5, ca2
    show "after csethigh, even to the same:   ", a5
    csc cs0, 0(cs0)
    clc ca2, 0(cs0)
    cgettag a5, ca2
    show "stored and loaded again:            ", a5
    cgetaddr a1, cs0
    sw a1, 0(cs0)
    clc ca2, 0(cs0)
    cgettag a5, ca2
    show "after an integer is stored over it: ", a5
    cgetaddr a5, ca2
    show "the address is still there:         ", a5
    ebreak

routines

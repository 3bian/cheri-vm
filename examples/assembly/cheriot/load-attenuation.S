/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a capability loaded through one without load-global or load-mutable */
/* comes back weaker. run it with --trap-policy stop */

.include "common.inc"

.globl _start
_start:
    start
    cspecialr cs0, mtdc
    li a0, 0x80002000
    csetaddr cs0, cs0, a0
    csetbounds cs0, cs0, 64
    csc cs0, 0(cs0)
    clc ca2, 0(cs0)
    cgetperm a5, ca2
    show "loaded through the full authority: ", a5
    li a1, ~0x02
    candperm ca3, cs0, a1
    clc ca2, 0(ca3)
    cgetperm a5, ca2
    show "through one without load-global:   ", a5
    li a1, ~0x08
    candperm ca3, cs0, a1
    clc ca2, 0(ca3)
    cgetperm a5, ca2
    show "through one without load-mutable:  ", a5
    cgettag a5, ca2
    show "the tag is kept:                   ", a5
    ebreak

routines

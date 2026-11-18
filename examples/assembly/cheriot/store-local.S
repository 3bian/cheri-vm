/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a local capability stored through an authority without store-local */
/* loses its tag; through one with it, it keeps it. run it with */
/* --trap-policy stop */

.include "common.inc"

.globl _start
_start:
    start
    cspecialr cs0, mtdc
    li a0, 0x80002000
    csetaddr cs0, cs0, a0
    csetbounds cs0, cs0, 64
    li a1, ~0x01
    candperm cs1, cs0, a1
    cgetperm a5, cs1
    show "a local capability, permissions:        ", a5
    csc cs1, 0(cs0)
    clc ca2, 0(cs0)
    cgettag a5, ca2
    show "stored with store-local, tag:           ", a5
    li a1, ~0x10
    candperm ca3, cs0, a1
    cgetperm a5, ca3
    show "an authority without it, permissions:   ", a5
    csc cs1, 8(ca3)
    clc ca2, 8(cs0)
    cgettag a5, ca2
    show "the local one stored through it, tag:   ", a5
    csc cs0, 16(ca3)
    clc ca2, 16(cs0)
    cgettag a5, ca2
    show "a global one stored through it, tag:    ", a5
    ebreak

routines

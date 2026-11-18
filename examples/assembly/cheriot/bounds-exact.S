/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a length that cannot be held exactly: csetboundsexact gives up the */
/* tag, csetbounds rounds up, and csetboundsrounddown rounds down. run it */
/* with --trap-policy stop */

.include "common.inc"

.globl _start
_start:
    start
    cspecialr cs0, mtdc
    li a0, 0x80002000
    csetaddr cs0, cs0, a0
    li a1, 0x1234567
    csetbounds ca2, cs0, a1
    cgetlen a5, ca2
    show "asked for 0x1234567; csetbounds gave:    ", a5
    cgettag a5, ca2
    show "tag:                                     ", a5
    csetboundsexact ca2, cs0, a1
    cgettag a5, ca2
    show "csetboundsexact, tag:                    ", a5
    csetboundsrounddown ca2, cs0, a1
    cgetlen a5, ca2
    show "csetboundsrounddown gave:                ", a5
    cgettag a5, ca2
    show "tag:                                     ", a5
    li a1, 0x100
    csetboundsexact ca2, cs0, a1
    cgettag a5, ca2
    show "csetboundsexact of 0x100, tag:           ", a5
    ebreak

routines

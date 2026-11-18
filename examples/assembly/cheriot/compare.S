/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* subsets and exact equality. run it with --trap-policy stop */

.include "common.inc"

.globl _start
_start:
    start
    cspecialr cs0, mtdc
    li a0, 0x80002000
    csetaddr cs0, cs0, a0
    csetbounds cs0, cs0, 64
    csetbounds ca2, cs0, 16
    ctestsubset a5, cs0, ca2
    show "the narrower is a subset of the wider:    ", a5
    ctestsubset a5, ca2, cs0
    show "the wider is a subset of the narrower:    ", a5
    li a1, ~0x04
    candperm ca3, cs0, a1
    ctestsubset a5, cs0, ca3
    show "fewer permissions is a subset:            ", a5
    ccleartag ca3, cs0
    ctestsubset a5, cs0, ca3
    show "one without a tag is not:                 ", a5
    cmove ca3, cs0
    csetequalexact a5, cs0, ca3
    show "a copy is exactly equal:                  ", a5
    ccleartag ca3, cs0
    csetequalexact a5, cs0, ca3
    show "the same bits without the tag are not:    ", a5
    cincoffset ca3, cs0, 4
    csetequalexact a5, cs0, ca3
    show "nor is one with another address:          ", a5
    ebreak

routines

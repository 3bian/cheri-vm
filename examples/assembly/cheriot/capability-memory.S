/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a capability stored and loaded with its tag, and the alignment both */
/* need. run it with --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    start
    handle handler
    cspecialr cs0, mtdc
    li a0, 0x80002000
    csetaddr cs0, cs0, a0
    csetbounds cs0, cs0, 64
    csc cs0, 8(cs0)
    clc ca2, 8(cs0)
    cgettag a5, ca2
    show "stored and loaded, tag:      ", a5
    cgetlen a5, ca2
    show "its length came with it:     ", a5
    lw a5, 8(cs0)
    show "the same memory as a word:   ", a5
    lw a5, 12(cs0)
    show "and the word above it:       ", a5
    print "a store to an address that is not a multiple of eight\n"
    csc cs0, 4(cs0)
    print "a load from one\n"
    clc ca2, 4(cs0)
    finish

.balign 4
handler:
    csrr a5, mcause
    show "  mcause ", a5
    csrr a5, mtval
    show "  mtval  ", a5
    skip
    mret

routines

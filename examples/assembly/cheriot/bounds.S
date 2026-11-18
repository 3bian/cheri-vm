/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* bounds set by a register and by an immediate; a read inside them, one */
/* at the last byte, and the fault from one past the end. run it with */
/* --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    start
    handle handler
    cspecialr cs0, mtdc
    li a0, 0x80002000
    csetaddr cs0, cs0, a0
    li a1, 64
    csetbounds ca2, cs0, a1
    cgetlen a5, ca2
    show "bounds from a register, length:   ", a5
    csetbounds ca2, cs0, 16
    cgetlen a5, ca2
    show "bounds from an immediate, length: ", a5
    li t0, 0x5a
    sb t0, 15(ca2)
    lbu a5, 15(ca2)
    show "the last byte inside reads:       ", a5
    print "a read one byte past the end\n"
    lbu a5, 16(ca2)
    print "a word that starts inside and ends outside\n"
    lw a5, 14(ca2)
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

/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* the trap registers are capabilities here. mtvec is given a handler */
/* bounded to its own nine instructions, mepc comes back as the capability */
/* that was running, and an integer written to one of them leaves no tag. */
/* run it with --trap-policy vector and --completion 0x80100000 */

.include "common.inc"

.globl _start
_start:
    start
    csrr s0, mtvec
    ytagr s1, s0
    show "mtvec at reset, its tag:              ", s1
    ylenr s1, s0
    show "and its length:                       ", s1
    point s0, handler
    ybndswi s0, s0, 36
    csrw mtvec, s0
    ecall
    show "in the handler, mcause was:           ", s4
    show "mepc had its tag:                     ", s6
    show "and the length of what was running:   ", s7
    show "the handler's own length, in bytes:   ", s8
    point s0, data
    ybndswi s0, s0, 16
    csrw mscratch, s0
    csrr s2, mscratch
    ytagr s1, s2
    show "mscratch, written with a capability:  ", s1
    li t0, 8
    csrs mscratch, t0
    csrr s2, mscratch
    ytagr s1, s2
    show "after csrs moves its address on by 8: ", s1
    show "the address:                          ", s2
    mv t0, s0
    csrw mscratch, t0
    csrr s2, mscratch
    ytagr s1, s2
    show "written with the address as a number: ", s1
    finish

.balign 4
handler:
    csrr s3, mepc
    csrr s4, mcause
    csrr s5, mtvec
    ytagr s6, s3
    ylenr s7, s3
    ylenr s8, s5
    yaddi s3, s3, 4
    csrw mepc, s3
    mret

.balign 16
data:
    .quad 0, 0

routines

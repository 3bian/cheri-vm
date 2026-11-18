/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a floating-point instruction with the unit off, in mstatus.FS, and the */
/* state one leaves dirty when it is on. run it with --trap-policy vector */
/* and --completion 0x80100000 */

.include "common.inc"
.option norvc

.macro state label
    csrr s1, mstatus
    srli s1, s1, 13
    andi s1, s1, 3
    show "\label", s1
.endm

.globl _start
_start:
    la t0, handler
    csrw mtvec, t0
    li t0, 0x6000
    csrc mstatus, t0
    state "FS, off:                     "
    print "an addition with it off\n"
    fadd.d fa0, fa0, fa0
    li t0, 0x2000
    csrs mstatus, t0
    state "FS, set to initial:          "
    csrr s1, fcsr
    state "after fcsr is only read:     "
    fmv.d.x fa0, zero
    state "after a register is written: "
    csrr s1, mstatus
    srli s1, s1, 63
    show "the summary bit, SD:         ", s1
    finish

.balign 4
handler:
    csrr s2, mcause
    show "  mcause ", s2
    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0
    mret

routines

/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* the software interrupt: msip set, taken, and cleared in the handler. */
/* run it with --trap-policy vector and --completion 0x80100000 */

.include "common.inc"

.globl _start
_start:
    la t0, handler
    csrw mtvec, t0
    li t0, 8
    csrs mie, t0
    csrsi mstatus, 8
    print "setting msip\n"
    li t0, CLINT
    li t1, 1
    sw t1, 0(t0)
    nop
    nop
    print "back in the program\n"
    finish

.balign 4
handler:
    csrr s2, mcause
    show "interrupt, mcause ", s2
    li t0, CLINT
    sw zero, 0(t0)
    mret

routines

/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a trap vector in vectored mode: an exception goes to its base, and an */
/* interrupt to the entry of its cause. run it with --trap-policy vector */
/* and --completion 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    la t0, vector
    addi t0, t0, 1
    csrw mtvec, t0
    ecall
    li t0, 8
    csrs mie, t0
    li t0, CLINT
    li t1, 1
    sw t1, 0(t0)
    csrsi mstatus, 8
    nop
    nop
    print "done\n"
    finish

.balign 4
vector:
    j exception
    j other
    j other
    j software
    j other
    j other
    j other
    j other

exception:
    print "an environment call came in at the base\n"
    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0
    mret

software:
    print "the software interrupt came in at entry 3\n"
    li t0, CLINT
    sw zero, 0(t0)
    mret

other:
    print "another entry\n"
    finish

routines

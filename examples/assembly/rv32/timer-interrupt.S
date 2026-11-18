/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a compare value, the enable bits, wfi, and the timer interrupt taken. */
/* run it with --trap-policy vector and --completion 0x80100000 */

.include "common.inc"

.globl _start
_start:
    la t0, handler
    csrw mtvec, t0
    li s0, CLINT
    li t1, 0xbff8
    add t1, t1, s0
    lw t2, 0(t1)
    addi t2, t2, 4
    li t1, 0x4000
    add t1, t1, s0
    sw t2, 0(t1)
    sw zero, 4(t1)
    li t0, 0x80
    csrs mie, t0
    print "waiting for the timer\n"
    csrsi mstatus, 8
1:  wfi
    beqz s1, 1b
    print "back in the program\n"
    finish

.balign 4
handler:
    csrr s2, mcause
    show "interrupt, mcause ", s2
    li t0, CLINT + 0x4000
    li t1, -1
    sw t1, 0(t0)
    sw t1, 4(t0)
    li s1, 1
    mret

routines

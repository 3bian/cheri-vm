/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a timer interrupt taken with interrupts on, and held off inside a */
/* sentry that turns them off. run it with --completion 0x80100000 */

.include "common.inc"

.globl _start
_start:
    start
    handle handler
    cspecialr cs0, mtdc
    li a0, CLINT
    csetaddr cs0, cs0, a0
    li a1, 0xc000
    csetbounds cs0, cs0, a1
    li a1, 0x4000
    cincoffset cs1, cs0, a1
    li a1, 0xbff8
    cincoffset cs0, cs0, a1
    li t0, 0x80
    csrs mie, t0
    lw t1, 0(cs0)
    addi t1, t1, 3
    sw t1, 0(cs1)
    sw zero, 4(cs1)
    print "waiting, with interrupts on\n"
    li a4, 0
    csrsi mstatus, 8
1:  beqz a4, 1b
    csrci mstatus, 8
    print "now through a sentry that turns them off\n"
    cllc ca1, busy
    cspecialr ca2, mscratchc
    li a0, 2
    csetaddr ca2, ca2, a0
    cseal ca1, ca1, ca2
    li a4, 0
    lw t1, 0(cs0)
    addi t1, t1, 3
    sw t1, 0(cs1)
    sw zero, 4(cs1)
    csrsi mstatus, 8
    cjalr cra, ca1
2:  beqz a4, 2b
    show "the handler had run when the sentry's code ended: ", a3
    show "and has run now that it has returned:             ", a4
    finish

busy:
    li t0, 2000
3:  addi t0, t0, -1
    bnez t0, 3b
    mv a3, a4
    cret

.balign 4
handler:
    li t1, -1
    sw t1, 0(cs1)
    sw t1, 4(cs1)
    li a4, 1
    mret

routines

/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a received character as an external interrupt: enabled at the UART and */
/* at the interrupt controller, then claimed and completed. run it with */
/* --trap-policy vector, --completion 0x80100000 and --uart in=/dev/stdin */

.include "common.inc"

.globl _start
_start:
    la t0, handler
    csrw mtvec, t0
    li t0, UART
    li t1, 1
    sw t1, 8(t0)
    li t0, CONTROLLER
    li t1, 2
    sw t1, 4(t0)
    li t0, 0x800
    csrs mie, t0
    print "type a character\n"
    csrsi mstatus, 8
1:  wfi
    beqz s1, 1b
    finish

.balign 4
handler:
    li s0, CONTROLLER
    lw s2, 8(s0)
    show "claimed source ", s2
    li t0, UART
    lw s3, 0(t0)
    show "the character  ", s3
    sw s2, 8(s0)
    li s1, 1
    mret

routines

/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a reservation lost to a trap between its two halves: the store after */
/* the trap fails. run it with --trap-policy vector and --completion */
/* 0x80100000 */

.include "common.inc"
.option norvc

.globl _start
_start:
    la t0, handler
    csrw mtvec, t0
    la s0, cell
    lr.d s1, (s0)
    addi s1, s1, 1
    sc.d s2, s1, (s0)
    show "sc.d straight after lr.d, 0 is success: ", s2
    lr.d s1, (s0)
    addi s1, s1, 1
    ecall
    sc.d s2, s1, (s0)
    show "sc.d with a trap in between:            ", s2
    ld s1, 0(s0)
    show "the cell:                               ", s1
    finish

.balign 4
handler:
    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0
    mret

.balign 8
cell:
    .quad 0

routines

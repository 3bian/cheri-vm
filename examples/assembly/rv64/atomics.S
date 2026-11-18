/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* A: a reservation kept and one lost, and the swap, add and logic forms, */
/* on words and doublewords */

.include "common.inc"

.globl _start
_start:
    la s0, cell
    li t0, 10
    sd t0, 0(s0)
    lr.d s1, (s0)
    addi s1, s1, 1
    sc.d s2, s1, (s0)
    show "sc.d after lr.d, 0 is success: ", s2
    sc.d s2, s1, (s0)
    show "sc.d with no reservation:      ", s2
    lr.w s1, (s0)
    la t1, other
    sc.w s2, s1, (t1)
    show "sc.w to another address:       ", s2
    ld s1, 0(s0)
    show "the cell now:                  ", s1
    li t0, 100
    amoswap.d s1, t0, (s0)
    show "amoswap.d gave the old value:  ", s1
    li t0, 5
    amoadd.d s1, t0, (s0)
    ld s1, 0(s0)
    show "after amoadd.d 5:              ", s1
    li t0, 0x0f
    amoand.w s1, t0, (s0)
    li t0, 0x30
    amoor.w s1, t0, (s0)
    li t0, 0xff
    amoxor.w s1, t0, (s0)
    lw s1, 0(s0)
    show "after and 0x0f, or 0x30, xor:  ", s1
    li t0, -5
    amomax.w s1, t0, (s0)
    amominu.w s1, t0, (s0)
    lw s1, 0(s0)
    show "after max -5 and minu -5:      ", s1
    li t0, -9
    amomin.d s1, t0, (s0)
    ld s1, 0(s0)
    show "after amomin.d -9:             ", s1
    ebreak

.balign 8
cell:
    .quad 0
other:
    .quad 0

routines

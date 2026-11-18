/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* Zicsr: read, write, set and clear, with a register and with an */
/* immediate; and the cycle and retired counters with their inhibit bits */

.include "common.inc"

.globl _start
_start:
    li s0, 0x12345678
    csrrw s1, mscratch, s0
    csrr s1, mscratch
    show "csrrw wrote:                 ", s1
    li s0, 0x0000ff00
    csrrs s1, mscratch, s0
    csrr s1, mscratch
    show "csrrs set 0x0000ff00:        ", s1
    csrrc s2, mscratch, s0
    show "csrrc returned the old value:", s2
    csrr s1, mscratch
    show "and cleared 0x0000ff00:      ", s1
    csrrwi s1, mscratch, 21
    csrr s1, mscratch
    show "csrrwi wrote 21:             ", s1
    csrrsi s1, mscratch, 10
    csrr s1, mscratch
    show "csrrsi set 10:               ", s1
    csrrci s1, mscratch, 1
    csrr s1, mscratch
    show "csrrci cleared 1:            ", s1
    csrr s2, minstret
    csrr s3, minstret
    sub s3, s3, s2
    show "minstret between two reads:  ", s3
    li s0, 5
    csrs mcountinhibit, s0
    csrr s2, minstret
    csrr s3, minstret
    sub s3, s3, s2
    show "the same with it inhibited:  ", s3
    csrr s2, mcycle
    csrr s3, mcycle
    sub s3, s3, s2
    show "mcycle, inhibited too:       ", s3
    ebreak

routines

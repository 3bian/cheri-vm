/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* Zba, Zbb, Zbc and Zbs, a few of each */

.include "common.inc"

.globl _start
_start:
    li s0, 0x00f0000000000100
    li s2, 5
    li s3, 0xffffffff00000003
    sh1add s1, s2, s0
    show "Zba sh1add, 5 * 2 + x:        ", s1
    sh3add s1, s2, s0
    show "Zba sh3add, 5 * 8 + x:        ", s1
    add.uw s1, s3, s2
    show "Zba add.uw, low word of 3:    ", s1
    clz s1, s0
    show "Zbb clz:                      ", s1
    ctz s1, s0
    show "Zbb ctz:                      ", s1
    cpop s1, s0
    show "Zbb cpop:                     ", s1
    li t0, -3
    max s1, s2, t0
    show "Zbb max of 5 and -3:          ", s1
    minu s1, s2, t0
    show "Zbb minu of the same:         ", s1
    rev8 s1, s0
    show "Zbb rev8:                     ", s1
    orc.b s1, s0
    show "Zbb orc.b:                    ", s1
    li t0, 8
    ror s1, s0, t0
    show "Zbb ror by 8:                 ", s1
    andn s1, s3, s2
    show "Zbb andn:                     ", s1
    sext.b s1, s0
    zext.h s4, s3
    add s1, s1, s4
    show "Zbb sext.b plus zext.h:       ", s1
    li t0, 0x87
    li t1, 0x1b
    clmul s1, t0, t1
    show "Zbc clmul of 0x87 and 0x1b:   ", s1
    clmulh s1, s0, s0
    show "Zbc clmulh:                   ", s1
    li t0, 40
    bset s1, zero, t0
    show "Zbs bset bit 40:              ", s1
    bclri s1, s0, 8
    show "Zbs bclri bit 8:              ", s1
    binvi s1, s0, 0
    show "Zbs binvi bit 0:              ", s1
    bexti s1, s0, 52
    show "Zbs bexti bit 52:             ", s1
    ebreak

routines

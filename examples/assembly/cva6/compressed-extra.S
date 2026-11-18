/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* Zcb: the further compressed forms, each beside the instruction it */
/* stands for */

.include "common.inc"

.macro pair
    bne a2, a3, 1f
    addi s5, s5, 1
1:  addi s6, s6, 1
.endm

.globl _start
_start:
    li s5, 0
    li s6, 0
    la s0, words
    li a4, 0x1234abcd
    sw a4, 0(s0)
    c.lbu a2, 1(s0)
    lbu a3, 1(s0)
    pair
    c.lhu a2, 2(s0)
    lhu a3, 2(s0)
    pair
    c.lh a2, 0(s0)
    lh a3, 0(s0)
    pair
    sw zero, 0(s0)
    c.sb a4, 1(s0)
    c.sh a4, 2(s0)
    lw a2, 0(s0)
    sb a4, 9(s0)
    sh a4, 10(s0)
    lw a3, 8(s0)
    pair
    mv a2, a4
    mv a3, a4
    c.zext.b a2
    andi a3, a3, 0xff
    pair
    mv a2, a4
    c.sext.b a2
    sext.b a3, a4
    pair
    mv a2, a4
    c.zext.h a2
    zext.h a3, a4
    pair
    mv a2, a4
    c.sext.h a2
    sext.h a3, a4
    pair
    li a2, -1
    li a3, -1
    c.zext.w a2
    zext.w a3, a3
    pair
    mv a2, a4
    c.not a2
    not a3, a4
    pair
    mv a2, a4
    mv a3, a4
    c.mul a2, a4
    mul a3, a3, a4
    pair
    show "pairs that agreed: ", s5
    show "pairs tried:       ", s6
    ebreak

.balign 8
words:
    .fill 4, 8, 0

routines

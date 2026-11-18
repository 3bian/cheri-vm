/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* loads and stores of a byte, a halfword and a word, signed and unsigned */

.include "common.inc"

.globl _start
_start:
    la s0, word
    li t0, 0x8180fffe
    sw t0, 0(s0)
    lw s1, 0(s0)
    show "lw:  ", s1
    lh s1, 0(s0)
    show "lh,  sign extended: ", s1
    lhu s1, 0(s0)
    show "lhu, zero extended: ", s1
    lb s1, 3(s0)
    show "lb  of the top byte: ", s1
    lbu s1, 3(s0)
    show "lbu of the top byte: ", s1
    li t0, 0x55
    sb t0, 1(s0)
    lw s1, 0(s0)
    show "after sb to byte 1: ", s1
    li t0, 0x1234
    sh t0, 2(s0)
    lw s1, 0(s0)
    show "after sh to byte 2: ", s1
    ebreak

.balign 4
word:
    .word 0

routines

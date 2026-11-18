/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* the compressed forms that exist only at 64 bits, each beside the */
/* instruction it stands for */

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
    la sp, words
    li a4, 0x123456789
    .option rvc
    c.sd a4, 8(s0)
    c.ld a2, 8(s0)
    .option norvc
    sd a4, 16(s0)
    ld a3, 16(s0)
    pair
    .option rvc
    c.sdsp a4, 24(sp)
    c.ldsp a2, 24(sp)
    .option norvc
    sd a4, 32(sp)
    ld a3, 32(sp)
    pair
    li a2, 0x7fffffff
    li a3, 0x7fffffff
    .option rvc
    c.addiw a2, 1
    .option norvc
    addiw a3, a3, 1
    pair
    .option rvc
    c.addw a2, a4
    .option norvc
    addw a3, a3, a4
    pair
    .option rvc
    c.subw a2, a4
    .option norvc
    subw a3, a3, a4
    pair
    .option rvc
    c.slli a2, 40
    c.srli a2, 33
    c.srai a2, 2
    .option norvc
    slli a3, a3, 40
    srli a3, a3, 33
    srai a3, a3, 2
    pair
    show "pairs that agreed: ", s5
    show "pairs tried:       ", s6
    ebreak

.balign 8
words:
    .fill 8, 8, 0

routines

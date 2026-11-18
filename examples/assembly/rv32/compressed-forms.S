/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* one of each kind of compressed instruction, beside the instruction it */
/* stands for; each pair must leave the same value */

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
    .option rvc
    c.li a2, 5
    .option norvc
    addi a3, zero, 5
    pair
    .option rvc
    c.addi a2, 3
    .option norvc
    addi a3, a3, 3
    pair
    .option rvc
    c.lui a2, 3
    .option norvc
    lui a3, 3
    pair
    .option rvc
    c.slli a2, 4
    .option norvc
    slli a3, a3, 4
    pair
    .option rvc
    c.srli a2, 2
    .option norvc
    srli a3, a3, 2
    pair
    .option rvc
    c.srai a2, 1
    .option norvc
    srai a3, a3, 1
    pair
    .option rvc
    c.andi a2, 0x1f
    .option norvc
    andi a3, a3, 0x1f
    pair
    li a4, 6
    .option rvc
    c.mv a2, a4
    .option norvc
    add a3, zero, a4
    pair
    .option rvc
    c.add a2, a4
    .option norvc
    add a3, a3, a4
    pair
    .option rvc
    c.sub a2, a4
    c.xor a2, a4
    c.or a2, a4
    c.and a2, a4
    .option norvc
    sub a3, a3, a4
    xor a3, a3, a4
    or a3, a3, a4
    and a3, a3, a4
    pair
    .option rvc
    c.sw a4, 4(s0)
    c.lw a2, 4(s0)
    .option norvc
    sw a4, 8(s0)
    lw a3, 8(s0)
    pair
    .option rvc
    c.swsp a4, 12(sp)
    c.lwsp a2, 12(sp)
    .option norvc
    sw a4, 16(sp)
    lw a3, 16(sp)
    pair
    .option rvc
    c.addi4spn a2, sp, 8
    .option norvc
    addi a3, sp, 8
    pair
    .option rvc
    c.addi16sp sp, 32
    c.mv a2, sp
    c.addi16sp sp, -32
    .option norvc
    addi a3, sp, 32
    pair
    li a2, 0
    li a3, 1
    .option rvc
    c.beqz a2, 2f
    li a3, 0
2:  c.bnez a3, 3f
    li a3, 0
3:  c.j 4f
    li a3, 0
4:  c.li a2, 1
    .option norvc
    pair
    show "pairs that agreed: ", s5
    show "pairs tried:       ", s6
    .option rvc
    c.ebreak
    .option norvc

.balign 4
words:
    .fill 16, 4, 0

routines

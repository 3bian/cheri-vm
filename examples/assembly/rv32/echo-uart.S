/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

.option norvc
.section .text
.globl _start
_start:
    lui x1, 0x10000
    addi x4, x0, 113
wait:
    lw x2, 4(x1)
    andi x2, x2, 1
    beq x2, x0, wait
    lw x3, 0(x1)
    beq x3, x4, done
    sw x3, 0(x1)
    jal x0, wait
done:
    ebreak

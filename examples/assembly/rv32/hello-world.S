/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a string written to the UART, one byte at a time, in a loop */

.include "common.inc"

.globl _start
_start:
    la a0, greeting
    li t0, UART
next:
    lbu t1, 0(a0)
    beqz t1, done
    sw t1, 0(t0)
    addi a0, a0, 1
    j next
done:
    ebreak

greeting:
    .asciz "Hello, world\n"

routines

/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* the UART, in 64-bit code: a string written a byte at a time */

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
    .asciz "Hello from 64 bits\n"

routines

/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* the 16550: its line status says the transmitter is empty, and a byte */
/* written to its transmit register is sent */

.include "common.inc"

.globl _start
_start:
    la a0, greeting
    li t0, UART
next:
    lbu t1, 0(a0)
    beqz t1, done
wait:
    lw t2, 0x14(t0)
    andi t2, t2, 0x20
    beqz t2, wait
    sw t1, 0(t0)
    addi a0, a0, 1
    j next
done:
    ebreak

greeting:
    .asciz "Hello from the CVA6\n"

routines

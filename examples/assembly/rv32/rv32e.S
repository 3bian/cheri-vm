/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* RV32E: sixteen registers, and the trap from naming a seventeenth. run */
/* it on the rv32e profile with --trap-policy vector and --completion */
/* 0x80100000 */

.equ UART, 0x10000000
.equ TOHOST, 0x80100000

.globl _start
_start:
    la t0, handler
    csrw mtvec, t0
    li a5, 'f'
    li t0, UART
    sw a5, 0(t0)
    li a0, '\n'
    sw a0, 0(t0)
    .word 0x00100813
    li t0, TOHOST
    li t1, 1
    sw t1, 0(t0)
    sw zero, 4(t0)
1:  j 1b

.balign 4
handler:
    csrr a0, mcause
    addi a0, a0, '0'
    li t0, UART
    sw a0, 0(t0)
    li a0, '\n'
    sw a0, 0(t0)
    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0
    mret

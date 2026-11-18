/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* Ethernet: a frame of 60 bytes sent, a word at a time. with no */
/* --ethernet option nothing is at the other end, so nothing is received */

.include "common.inc"

.globl _start
_start:
    li s0, ETHERNET
    lw s1, 0(s0)
    show "link:                  ", s1
    li t0, 60
    sw t0, 8(s0)
    li t0, 0xffffffff
    sw t0, 12(s0)
    li t0, 0x0000ffff
    sw t0, 12(s0)
    li t0, 0x00000002
    sw t0, 12(s0)
    li t0, 0x08060001
    sw t0, 12(s0)
    li t1, 11
1:  sw zero, 12(s0)
    addi t1, t1, -1
    bnez t1, 1b
    li t0, 1
    sw t0, 16(s0)
    lw s1, 4(s0)
    show "status after the send: ", s1
    lw s1, 20(s0)
    show "length received:       ", s1
    ebreak

routines

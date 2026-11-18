/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* I2C: two bytes written to the EEPROM at 0x50, and the read that */
/* returns them */

.include "common.inc"

.globl _start
_start:
    li s0, I2C
    li t0, 0x50
    sw t0, 0(s0)
    li t0, 16
    sw t0, 4(s0)
    li s2, 1
    sw s2, 8(s0)
    li t0, 0xab
    sw t0, 4(s0)
    sw s2, 8(s0)
    li t0, 0xcd
    sw t0, 4(s0)
    sw s2, 8(s0)
    li t0, 3
    sw t0, 8(s0)
    lw s1, 12(s0)
    show "status after the write:  ", s1
    li t0, 16
    sw t0, 4(s0)
    sw s2, 8(s0)
    li s3, 2
    sw s3, 8(s0)
    lw s1, 4(s0)
    show "the first byte read:     ", s1
    sw s3, 8(s0)
    lw s1, 4(s0)
    show "the second:              ", s1
    li t0, 3
    sw t0, 8(s0)
    li t0, 0x51
    sw t0, 0(s0)
    sw s2, 8(s0)
    lw s1, 12(s0)
    show "status, nothing at 0x51: ", s1
    ebreak

routines

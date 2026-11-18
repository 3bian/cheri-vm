/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* GPIO, SPI, I2C, storage and Ethernet, each through a capability */
/* bounded to its registers. run it with --trap-policy stop */

.include "common.inc"

.globl _start
_start:
    start
    device cs0, GPIO, 20
    li t0, 0x0f
    sw t0, 8(cs0)
    li t0, 0x35
    sw t0, 4(cs0)
    lw a5, 0(cs0)
    show "GPIO, the pins driven:           ", a5
    device cs0, SPI, 24
    sw zero, 12(cs0)
    sw zero, 8(cs0)
    li t0, 0x95
    sw t0, 0(cs0)
    lw a5, 0(cs0)
    show "SPI, 0x95 out and back:          ", a5
    device cs0, I2C, 24
    li t0, 0x50
    sw t0, 0(cs0)
    li t0, 16
    sw t0, 4(cs0)
    li a2, 1
    sw a2, 8(cs0)
    li t0, 0xab
    sw t0, 4(cs0)
    sw a2, 8(cs0)
    li t0, 3
    sw t0, 8(cs0)
    li t0, 16
    sw t0, 4(cs0)
    sw a2, 8(cs0)
    li t0, 2
    sw t0, 8(cs0)
    lw a5, 4(cs0)
    show "I2C, the byte the EEPROM kept:   ", a5
    device cs0, STORAGE, 24
    li t0, 0x12345678
    sw t0, 8(cs0)
    li t0, 2
    sw t0, 12(cs0)
    sw zero, 8(cs0)
    li t0, 1
    sw t0, 12(cs0)
    lw a5, 8(cs0)
    show "storage, the word read back:     ", a5
    device cs0, ETHERNET, 36
    li t0, 60
    sw t0, 8(cs0)
    li t1, 15
1:  sw zero, 12(cs0)
    addi t1, t1, -1
    bnez t1, 1b
    li t0, 1
    sw t0, 16(cs0)
    lw a5, 4(cs0)
    show "Ethernet, status after a send:   ", a5
    ebreak

routines

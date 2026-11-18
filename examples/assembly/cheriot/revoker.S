/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* the hardware revoker: a sweep that clears the tag of a marked */
/* capability where it lies in memory, the epoch that counts sweeps, and */
/* the interrupt at the end of one. run it on the cheriot-hwr profile */
/* with --completion 0x80100000 */

.include "common.inc"

.globl _start
_start:
    start
    handle handler
    cspecialr cs0, mtdc
    li a0, 0x80008000
    csetaddr cs0, cs0, a0
    csetbounds cs0, cs0, 64
    cspecialr cs1, mtdc
    li a0, 0x80009000
    csetaddr cs1, cs1, a0
    csetbounds cs1, cs1, 16
    csc cs0, 0(cs1)
    cspecialr ca2, mtdc
    li a0, 0x83000200
    csetaddr ca2, ca2, a0
    csetbounds ca2, ca2, 4
    li t0, 1
    sw t0, 0(ca2)
    device cs0, REVOKER, 24
    lw a5, 12(cs0)
    show "the epoch before:                   ", a5
    li t0, 0x80009000
    sw t0, 0(cs0)
    li t0, 0x80009010
    sw t0, 4(cs0)
    li t0, 1
    sw t0, 8(cs0)
1:  lw a5, 12(cs0)
    andi t0, a5, 1
    bnez t0, 1b
    show "the epoch after one sweep:          ", a5
    sw zero, 0(ca2)
    clc ca1, 0(cs1)
    cgettag a5, ca1
    show "the mark is gone, yet the tag is:   ", a5
    device ca3, CONTROLLER, 16
    li t0, 0x80
    sw t0, 4(ca3)
    li t0, 0x800
    csrs mie, t0
    li t0, 1
    sw t0, 20(cs0)
    sw t0, 8(cs0)
    print "a second sweep, with its interrupt asked for\n"
    li a4, 0
    csrsi mstatus, 8
2:  beqz a4, 2b
    show "the handler claimed source:         ", a1
    lw a5, 12(cs0)
    show "the epoch now:                      ", a5
    finish

.balign 4
handler:
    lw a1, 8(ca3)
    sw a1, 8(ca3)
    li a4, 1
    mret

routines

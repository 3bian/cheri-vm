/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* the stack high-water mark: a store below the mark, through the stack */
/* pointer's capability, moves it down. run it with --trap-policy stop */

.include "common.inc"

.equ MSHWM, 0xbc1
.equ MSHWMB, 0xbc2

.globl _start
_start:
    start
    cspecialr csp, mtdc
    li a0, 0x80004000
    csetaddr csp, csp, a0
    li a1, 0x1000
    csetbounds csp, csp, a1
    csrw MSHWMB, a0
    li a0, 0x80005000
    csrw MSHWM, a0
    csetaddr csp, csp, a0
    csrr a5, MSHWM
    show "the mark, set to the top of the stack: ", a5
    sw zero, -16(csp)
    csrr a5, MSHWM
    show "after a store 16 bytes down:           ", a5
    sw zero, -256(csp)
    csrr a5, MSHWM
    show "after one 256 bytes down:              ", a5
    sw zero, -32(csp)
    csrr a5, MSHWM
    show "a store above the mark leaves it:      ", a5
    csrr a5, MSHWMB
    show "the base it stops at:                  ", a5
    ebreak

routines

/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* Zifencei: code that changes an instruction ahead of itself, and fence.i */
/* so that the new instruction is the one that runs */

.include "common.inc"
.option norvc

.globl _start
_start:
    la t0, replacement
    lw t1, 0(t0)
    la t0, changed
    sw t1, 0(t0)
    fence.i
changed:
    li s2, 1
    show "the instruction that ran loaded ", s2
    ebreak

replacement:
    li s2, 2

routines

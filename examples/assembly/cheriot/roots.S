/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* the three roots a program starts with: the bounds and permissions of */
/* the program counter's, the memory root in mtdc and the sealing root in */
/* mscratchc. run it with --trap-policy stop */

.include "common.inc"

.macro describe name, register
    cgetbase a5, \register
    show "\name base:        ", a5
    cgettop a5, \register
    show "\name top:         ", a5
    cgetperm a5, \register
    show "\name permissions: ", a5
    cgettag a5, \register
    show "\name tag:         ", a5
.endm

.globl _start
_start:
    start
    auipcc cs0, 0
    describe "pcc      ", cs0
    cspecialr cs0, mtdc
    describe "mtdc     ", cs0
    cspecialr cs0, mscratchc
    describe "mscratchc", cs0
    ebreak

routines

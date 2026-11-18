/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* every field read out of one capability: address, base, top, length, */
/* permissions, type, tag and the high word. run it with --trap-policy */
/* stop */

.include "common.inc"

.globl _start
_start:
    start
    cspecialr cs0, mtdc
    li a0, 0x80001000
    csetaddr cs0, cs0, a0
    csetbounds cs0, cs0, 0x200
    cincoffset cs0, cs0, 0x10
    cgetaddr a5, cs0
    show "address:     ", a5
    cgetbase a5, cs0
    show "base:        ", a5
    cgettop a5, cs0
    show "top:         ", a5
    cgetlen a5, cs0
    show "length:      ", a5
    cgetperm a5, cs0
    show "permissions: ", a5
    cgettype a5, cs0
    show "type:        ", a5
    cgettag a5, cs0
    show "tag:         ", a5
    cgethigh a5, cs0
    show "high word:   ", a5
    ebreak

routines

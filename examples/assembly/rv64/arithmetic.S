/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* 64-bit arithmetic, and the word forms with their sign extension */

.include "common.inc"

.globl _start
_start:
    li s0, 0x7fffffff
    li s1, 1
    add s2, s0, s1
    show "add  0x7fffffff + 1:            ", s2
    addw s2, s0, s1
    show "addw of the same, sign extended:", s2
    li s0, 0x100000000
    sub s2, s0, s1
    show "sub  0x100000000 - 1:           ", s2
    subw s2, s0, s1
    show "subw sees only the low word:    ", s2
    li s0, 1
    slli s2, s0, 40
    show "slli by 40:                     ", s2
    slliw s2, s0, 31
    show "slliw by 31, sign extended:     ", s2
    li s0, -16
    srli s2, s0, 60
    show "srli -16 by 60:                 ", s2
    srai s2, s0, 60
    show "srai -16 by 60:                 ", s2
    srliw s2, s0, 28
    show "srliw -16 by 28:                ", s2
    li s0, 0xffffffff
    addiw s2, s0, 0
    show "addiw 0xffffffff, 0 (sext.w):   ", s2
    ebreak

routines

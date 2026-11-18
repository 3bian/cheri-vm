/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* the 16550 through a capability. no register holds one at reset, so the */
/* program makes its own from the program counter: an address, bounds of */
/* 32 bytes, and then a store through it */

.include "common.inc"

.globl _start
_start:
    start
    print "Hello from a machine where every pointer is a capability\n"
    ytagr s0, gp
    show "the UART capability, its tag: ", s0
    ybaser s0, gp
    show "its base:                     ", s0
    ylenr s0, gp
    show "its length:                   ", s0
    ypermr s0, gp
    show "its permissions:              ", s0
    li s0, UART
    ytagr s0, s0
    show "the same address as a number: ", s0
    ebreak

routines

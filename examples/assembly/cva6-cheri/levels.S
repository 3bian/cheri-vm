/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* levels, from zylg: a capability is global or local, and a local one */
/* keeps its tag in memory only when stored through a capability with */
/* store-local. a load through one without load-global gives a local copy */

.include "common.inc"

.equ LOAD_GLOBAL, 1 << 2
.equ STORE_LOCAL, 1 << 3
.equ GLOBAL, 1 << 4

.globl _start
_start:
    start
    point s0, slots
    ybndswi s0, s0, 64
    point s1, value
    ybndswi s1, s1, 8
    li t0, GLOBAL
    ypermc s2, s1, t0
    ypermr s3, s1
    show "a global capability:                   ", s3
    ypermr s3, s2
    show "a local one, the global bit gone:      ", s3
    li t0, STORE_LOCAL
    ypermc s4, s0, t0
    sy s2, 0, s0
    ly s5, 0, s0
    ytagr s3, s5
    show "local, stored with store-local, tag:   ", s3
    sy s2, 16, s4
    ly s5, 16, s0
    ytagr s3, s5
    show "local, stored without it, tag:         ", s3
    sy s1, 32, s4
    ly s5, 32, s0
    ytagr s3, s5
    show "global, stored without it, tag:        ", s3
    li t0, LOAD_GLOBAL
    ypermc s6, s0, t0
    ly s5, 32, s6
    ytagr s3, s5
    show "that one, loaded without load-global:  ", s3
    ypermr s3, s5
    show "and its permissions, now a local's:    ", s3
    ebreak

.balign 16
slots:
    .fill 8, 8, 0
value:
    .quad 0

routines

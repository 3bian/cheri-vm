/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* every comparison, and the link register of jal and jalr */

.include "common.inc"

.globl _start
_start:
    li s0, -1
    li s1, 1
    print "beq on equal values, taken: "
    li a0, 1
    beq s0, s0, 1f
    li a0, 0
1:  call put_hex
    call put_line
    print "bne on different values, taken: "
    li a0, 1
    bne s0, s1, 1f
    li a0, 0
1:  call put_hex
    call put_line
    print "blt -1 < 1, signed, taken: "
    li a0, 1
    blt s0, s1, 1f
    li a0, 0
1:  call put_hex
    call put_line
    print "bltu 0xffffffff < 1, unsigned, taken: "
    li a0, 1
    bltu s0, s1, 1f
    li a0, 0
1:  call put_hex
    call put_line
    print "bge 1 >= -1, signed, taken: "
    li a0, 1
    bge s1, s0, 1f
    li a0, 0
1:  call put_hex
    call put_line
    print "bgeu 0xffffffff >= 1, unsigned, taken: "
    li a0, 1
    bgeu s0, s1, 1f
    li a0, 0
1:  call put_hex
    call put_line
at_jal:
    jal s2, after_jal
after_jal:
    la s3, at_jal
    sub s2, s2, s3
    show "jal links to the instruction after it, bytes on: ", s2
    la s4, target
at_jalr:
    jalr s2, 0(s4)
target:
    la s3, at_jalr
    sub s2, s2, s3
    show "jalr links to the instruction after it, bytes on: ", s2
    ebreak

routines

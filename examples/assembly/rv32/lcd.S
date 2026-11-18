/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

.option norvc
.section .text
.globl _start
_start:
  li s0, 0x10001000
  li s1, 0x10003000
  li t0, 3
  sw t0, 8(s1)
  li t0, 2
  sw t0, 4(s1)
  sw zero, 8(s0)
  la s2, setup
  la s3, setup_end
next:
  lbu t0, 0(s2)
  lbu t1, 1(s2)
  ori t0, t0, 2
  sw t0, 4(s1)
  sw t1, 0(s0)
  addi s2, s2, 2
  bne s2, s3, next
  li t0, 3
  sw t0, 4(s1)
  la s2, colors
  li s4, 4
band:
  lbu t1, 0(s2)
  lbu t2, 1(s2)
  li s5, 16
pixel:
  sw t1, 0(s0)
  sw t2, 0(s0)
  addi s5, s5, -1
  bnez s5, pixel
  addi s2, s2, 2
  addi s4, s4, -1
  bnez s4, band
  li t0, -1
  sw t0, 8(s0)
  ebreak
setup:
  .byte 0, 0x01, 0, 0x11, 0, 0x3a, 1, 0x05, 0, 0x29
  .byte 0, 0x2a, 1, 0, 1, 0, 1, 0, 1, 7
  .byte 0, 0x2b, 1, 0, 1, 0, 1, 0, 1, 7
  .byte 0, 0x2c
setup_end:
colors:
  .byte 0xf8, 0x00, 0x07, 0xe0, 0x00, 0x1f, 0xff, 0xff

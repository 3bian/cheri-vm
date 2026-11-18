/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

.option norvc
.section .text
.globl _start
_start:
  li s0, 0x10002000
  li s1, 0x10000000
  li t0, 0x48
  sw t0, 0(s0)
  li t0, 0x10000
  sw t0, 16(s0)
  sw zero, 4(s0)
  li t0, 1
  sw t0, 8(s0)
  li t0, 2
  sw t0, 8(s0)
  lw a0, 4(s0)
  jal ra, hex
  li t0, 2
  sw t0, 8(s0)
  lw a0, 4(s0)
  jal ra, hex
  li t0, 3
  sw t0, 8(s0)
  li t0, 10
  sw t0, 0(s1)
  ebreak
hex:
  srli a1, a0, 4
  andi a1, a1, 15
  jal t1, digit
  andi a1, a0, 15
  jal t1, digit
  jalr zero, 0(ra)
digit:
  li t0, 10
  blt a1, t0, decimal
  addi a1, a1, 39
decimal:
  addi a1, a1, 48
  sw a1, 0(s1)
  jalr zero, 0(t1)

/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a temperature sensor on I2C, read four times a second and printed */
/* when the reading changes, until a `q` arrives on the UART */

.option norvc
.section .text
.globl _start
_start:
  li s0, 0x10002000
  li s1, 0x10000000
  li s2, 0x0200bff8
  li s3, -1
read:
  li t0, 0x48
  sw t0, 0(s0)
  li t0, 0x10000
  sw t0, 16(s0)
  sw zero, 4(s0)
  li t0, 1
  sw t0, 8(s0)
  li t0, 2
  sw t0, 8(s0)
  lw s4, 4(s0)
  li t0, 2
  sw t0, 8(s0)
  lw s5, 4(s0)
  li t0, 3
  sw t0, 8(s0)
  slli t0, s4, 8
  or t0, t0, s5
  beq t0, s3, same
  mv s3, t0
  mv a0, s4
  jal ra, hex
  mv a0, s5
  jal ra, hex
  li t0, 10
  sw t0, 0(s1)
same:
  lw s6, 0(s2)
  addi s6, s6, 500
  li t0, 0x02004000
  sw s6, 0(t0)
  sw zero, 4(t0)
wait:
  wfi
  lw t0, 4(s1)
  andi t0, t0, 1
  beqz t0, quiet
  lw t0, 0(s1)
  li t1, 113
  beq t0, t1, done
quiet:
  lw t0, 0(s2)
  sub t0, t0, s6
  bltz t0, wait
  j read
done:
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

/* Copyright 3bian Limited and Contributors */
/* SPDX-License-Identifier: MIT */

/* a display on SPI and two GPIO pins: four bands of color that move up *
/* a band every quarter of a second, until a `q` arrives on the UART */

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
  li s6, 0
  li s7, 0x0200bff8
  li s8, 0x10000000
frame:
  li s4, 0
band:
  add t0, s4, s6
  andi t0, t0, 3
  slli t0, t0, 1
  la s2, colors
  add s2, s2, t0
  lbu t1, 0(s2)
  lbu t2, 1(s2)
  li s5, 5120
pixel:
  sw t1, 0(s0)
  sw t2, 0(s0)
  addi s5, s5, -1
  bnez s5, pixel
  addi s4, s4, 1
  li t0, 4
  bne s4, t0, band
  lw s9, 0(s7)
  addi s9, s9, 500
  li t0, 0x02004000
  sw s9, 0(t0)
  sw zero, 4(t0)
wait:
  wfi
  lw t0, 4(s8)
  andi t0, t0, 1
  beqz t0, quiet
  lw t0, 0(s8)
  li t1, 113
  beq t0, t1, done
quiet:
  lw t0, 0(s7)
  sub t0, t0, s9
  bltz t0, wait
  addi s6, s6, 1
  j frame
done:
  li t0, -1
  sw t0, 8(s0)
  ebreak
setup:
  .byte 0, 0x01, 0, 0x11, 0, 0x3a, 1, 0x05, 0, 0x29
  .byte 0, 0x2a, 1, 0, 1, 0, 1, 0, 1, 159
  .byte 0, 0x2b, 1, 0, 1, 0, 1, 0, 1, 127
  .byte 0, 0x2c
setup_end:
colors:
  .byte 0xf8, 0x00, 0x07, 0xe0, 0x00, 0x1f, 0xff, 0xff

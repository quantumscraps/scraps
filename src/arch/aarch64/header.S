.section ".text._start"

.global _start

_start:
	add x13, x18, #0x16 // creates the "MZ" magic
	b __early_entry     // branch to rest of code
	.quad __start       // kernel entry point
	.quad __kernel_size // kernel size
	.quad 0b1010        // Little Endian, 4K pages
	.quad 0             // reserved
	.quad 0             // reserved
	.quad 0             // reserved
	.ascii "ARM\x64"    // arm64 Image magic
	.long 0             // reserved
__early_entry:
	mrs	x5, mpidr_el1   // processor ID info
	and	x5, x5, #3      // only get last 2 bits
	cbz	x5, 2f          // branch to `2` if processor 0
1:	wfe                     // else stay in this wfe loop
	b	1b
2:
	ldr	x5, =_start     // set up stack before kernel image
	mov	sp, x5

	ldr	x5, =__bss_start
	ldr	w6, =__bss_size // w6 has bss_size
3:	cbz	w6, 4f          // if it's zero go to `4`
	str	xzr, [x5], #8   // if not then clear it
	sub	w6, w6, #1      // decrement by 1
	cbnz	w6, 3b          // go back
4:	bl	kernel_init     // jump to kernel_init
	b	1b              // go back to wfe loop if fail

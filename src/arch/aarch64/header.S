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
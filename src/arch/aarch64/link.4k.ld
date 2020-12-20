ENTRY(_start)

SECTIONS
{
	. = 0xffff0000_00000000;
	/* 48 bit VA, just dump our kernel at the start for now */
	__start = .;
	__ro_start = .;
	.text :
	{
		*(.text._start) *(.text*)
	}

	.rodata :
	{
		*(.rodata*)
	}
	. = ALIGN(4096);
	__ro_end = .;
	.data :
	{
		*(.data*)
	}
	.bss ALIGN(8):
	{
		__bss_start = .;
		*(.bss*);
		. = ALIGN(8);
		. += 8;
		__bss_end = .;
	}
	__bss_size = __bss_end - __bss_start;
	/* align to 8 because we clear out bss in u64 chunks */
	. = ALIGN(4096);
	/* page align the end too!!! */
	__end = .;
	__heap_start = .;
	/DISCARD/ : { *(.comment*) *(.gnu) *(.note) *(.eh_frame*)}
}
__kernel_size = __end - __start;

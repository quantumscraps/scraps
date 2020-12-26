ENTRY(_start)

SECTIONS
{
	. = 0x80000;
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
	. = ALIGN(0x10000);
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
	. = ALIGN(0x10000);
	__end = .;
	__heap_start = .;
	/DISCARD/ : { *(.comment*) *(.gnu) *(.note) *(.eh_frame*)}
}
__kernel_size = __end - __start;

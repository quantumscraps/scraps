ENTRY(_header)

SECTIONS
{
	. = 0x80000;
	__start = .;
	.text :
	{

		*(.text._header) *(.text*)
	}

	.rodata :
	{
		*(.rodata*)
	}

	.data :
	{
		*(.data*)
	}

	.bss ALIGN(8):
	{
		__bss_start = .;
		*(.bss*);
		. = ALIGN(8);
		__bss_end = .;
	}
	__bss_size = __bss_end - __bss_start;
	/* align to 8 because we clear out bss in u64 chunks */
	__end = .;
	/DISCARD/ : { *(.comment*) *(.gnu) *(.note) *(.eh_frame*)}
}
__kernel_size = __end - __start;

ENTRY(_start)

SECTIONS
{
    . = 0x80000000;
    .text : {
        PROVIDE(__text_start = .);
        *(.text.init) *(.text .text.*)
        PROVIDE(__text_end = .);
    }
    PROVIDE(_global_pointer = .);
    .rodata : {
        PROVIDE(__rodata_start = .);
        *(.rodata .rodata.*)
        PROVIDE(__rodata_end = .);
    }
    .data : {
        . = ALIGN(4096);
        PROVIDE(__data_start = .);
        *(.sdata .sdata.*) *(.data .data.*)
        PROVIDE(__data_end = .);
    }
    .bss : {
        . = ALIGN(8);
        PROVIDE(__bss_start = .);
        *(.sbss .sbss.*) *(.bss .bss.*)
        PROVIDE(__bss_end = .);
    }
    PROVIDE(__stack = _bss_end + 0x80000);
    PROVIDE(__heap_start = _stack);
}
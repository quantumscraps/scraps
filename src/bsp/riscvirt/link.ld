ENTRY(_start)

SECTIONS
{
    . = 0x80000000;
    .text : {
        PROVIDE(_text_start = .);
        *(.text.init) *(.text .text.*)
        PROVIDE(_text_end = .);
    }
    PROVIDE(_global_pointer = .);
    .rodata : {
        PROVIDE(_rodata_start = .);
        *(.rodata .rodata.*)
        PROVIDE(_rodata_end = .);
    }
    .data : {
        . = ALIGN(4096);
        PROVIDE(_data_start = .);
        *(.sdata .sdata.*) *(.data .data.*)
        PROVIDE(_data_end = .);
    }
    .bss : {
        . = ALIGN(8);
        PROVIDE(_bss_start = .);
        *(.sbss .sbss.*) *(.bss .bss.*)
        PROVIDE(_bss_end = .);
    }
    PROVIDE(_stack = _bss_end + 0x80000);
    PROVIDE(_heap_start = _stack);
    PROVIDE(_heap_size = _memory_end - _stack);
}
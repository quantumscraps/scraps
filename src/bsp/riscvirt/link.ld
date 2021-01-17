ENTRY(_start)

SECTIONS
{
    . = 0x80000000;
    PROVIDE(__kern_start = .);
    .text : {
        PROVIDE(__text_start = .);
        /* HACK: Apparently QEMU disregards ENTRY and just jumps to 0x80000000, so we have to put rpt at the end */
        *(.text.init)
        . = ALIGN(4);
        *(.text.trap)
        *(.text .text.*)
        PROVIDE(__text_end = .);
    }
    PROVIDE(__global_pointer = .);
    .rodata : {
        PROVIDE(__rodata_start = .);
        *(.rodata .rodata.*)
        PROVIDE(__rodata_end = .);
    }
    .data : {
        . = ALIGN(4096);
        PROVIDE(__data_start = .);
        *(.sdata .sdata.*) *(.data)
        PROVIDE(__data_end = .);
    }
    .bss : {
        . = ALIGN(8);
        PROVIDE(__bss_start = .);
        *(.sbss .sbss.*) *(.bss .bss.*)
        PROVIDE(__bss_end = .);
        . = ALIGN(8);
    }
    PROVIDE(__stack = __bss_end + 0x80000);
    PROVIDE(__heap_start = __stack);
    . = __heap_start + 0x100000;
    . = ALIGN(4096);
    .data.rpt : {
        *(.data.rpt)
    }
    PROVIDE(__kern_end = .);
}
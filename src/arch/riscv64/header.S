.section .data

.section .text.init
.global _start
_start:
    la ra, __early_entry
    la gp, __global_pointer
    la sp, __stack
    ret
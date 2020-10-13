.section .data

.section .text.init
.global _start
_start:
    # Only run on hart zero
    csrr t0, mhartid
    bnez t0, 3f
    # Set privilege level to machine-mode
    csrw satp, zero
.option push
.option norelax
    la gp, __global_pointer

    # Zero out BSS
    la a0, __bss_start
    la a1, __bss_end
    bgeu a0, a1, 2f
1:
    sd zero, (a0)
    addi a0, a0, 8
    bltu a0, a1, 1b
2:
    la sp, _stack # setup stack pointer
    # enable interrupts basically
    li		t0, (0b11 << 11) | (1 << 7) | (1 << 3)
    csrw	mstatus, t0
    # location to jump to
    la		t1, kernel_init
    csrw	mepc, t1
    # trap vector
    la		t2, asm_trap_vector
    csrw	mtvec, t2
    # interrupt enable
    # 1 << 3 = software interrupt (M-mode)
    # 1 << 7 = timer interrupt
    # 1 << 11 = external interrupt
    li		t3, (1 << 3) | (1 << 7) | (1 << 11)
    csrw	mie, t3
    # Return to wfi loop
    la		ra, 4f
    # perform return
    mret
4:
	wfi
	j	4b
.option pop

# Location where non-zero harts go (secondary harts)
# currently a wfi loop
3:
    wfi
    j 3b

# Trap vector also included here for now
.global asm_trap_vector
asm_trap_vector:
    # don't handle traps, just return
    mret
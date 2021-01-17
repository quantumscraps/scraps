.section .data

.section .text.init
.global _start
.extern __root_page_table

# _start's goal is to setup higher half paging as fast as possible
# then jump to __early_entry
# Don't clobber a0 (hart id) or a1 (DTB ptr)
_start:
#     # First check start id, otherwise loop
#     csrr    t0, mhartid
#     bnez    t0, 4f

#     # Figure out __kern_start's index (divide by 1GiB)
#     # which we turn into a shift
#     la      t0, __kern_start
#     # folding (x >> 30) << 3 = x >> 27
#     # (x / ONEGIG) * 8
#     srli    t3, t0, 27 # t3 = index
    
#     # TODO: Handle cases where the kernel is larger than one gigapage.

#     # Setup page entry
#     #               ~~~~~~~~ valid
#     #                          ~~~~~~~~~~~~ RWX
#     #                                         ~~~~~~~~ global
#     mv      t1, t0
#     # Mask out lower 1g
#     li      t2, 0xffffffffc0000000
#     and     t1, t1, t2
#     # folding (x >> 12) << 10 = x >> 2
#     srli    t1, t1, 2
#     ori     t1, t1, (1 << 0) | (0b111 << 1) | (1 << 5)

#     # Write entry at identity map index, and also higher half index (3)
#     la      t2, __root_page_table
#     # 8 * 3
#     sd      t1, 24(t2)
#     add     t2, t2, t3
#     sd      t1, 0(t2)
#     # sub     t2, t2, t3
#     # ~~# no need to sub since we are shifting right anyways~~

#     # setup satp
#     li      t4, 8
#     slli    t4, t4, 60
#     srli    t5, t2, 12
#     or      t4, t4, t5
#     csrw    satp, t4
#     sfence.vma

#     # setup mstatus and mret to new virtual address
#     li      t5, 1 << 5 # MPP = S-mode
#     csrw    mstatus, t5

#     # jump to next part but with new address
#     la      t4, __unmap_identity
#     sub     t4, t4, t0
#     li      t5, 0xC0000000
#     add     t4, t4, t5
#     csrw    mepc, t5

# __unmap_identity:
    la ra, __early_entry
    la gp, __global_pointer
    la sp, __stack
    ret

# 4f:
#     wfi
#     j 4f
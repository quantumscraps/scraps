use register::{mmio::*, register_bitfields, InMemoryRegister};

register_bitfields! {
    u64,
    // Sv39 Page Table Entry
    PTE_39 [
        // Physical Page Number 2
        PPN2 OFFSET(28) NUMBITS(26) [],
        // Physical Page Number 1
        PPN1 OFFSET(19) NUMBITS(9) [],
        // Physical Page Number 0
        PPN0 OFFSET(10) NUMBITS(9) [],
        // Dirty Bit
        D OFFSET(7) NUMBITS(1) [],
        // Accessed Bit
        A OFFSET(6) NUMBITS(1) [],
        // Global Mapping Bit
        G OFFSET(5) NUMBITS(1) [],
        // User mode bit
        U OFFSET(4) NUMBITS(1) [],
        // Permissions
        XWR OFFSET(1) NUMBITS(3) [
            Pointer = 0b000,
            ReadOnly = 0b001,
            ReadWrite = 0b011,
            ExecOnly = 0b100,
            ReadExec = 0b101,
            ReadWriteExec = 0b111
        ],
        // Valid bit
        V OFFSET(0) NUMBITS(1) []
    ]
}

#[inline(never)]
pub const unsafe fn init() {}

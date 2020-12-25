use cortex_a::{barrier, regs::*};
use register::{mmio::*, register_bitfields, InMemoryRegister};

register_bitfields! {
    u64,
    // ARMv8 table descriptor
    TABLE_DESC [
        // basically the NX bit, so user can't execute here
        UXN OFFSET(54) NUMBITS(0b1) [
            False = 0,
            True = 1
        ],
        // ditto but for EL1 even
        PXN OFFSET(53) NUMBITS(0b1) [
            False = 0,
            True = 1
        ],
        // Contiguous
        CONT OFFSET(52) NUMBITS(0b1) [
            False = 0,
            True = 1
        ],
        //Actual VA (32 bits not including the 16 bit in page offset)
        ADDR OFFSET(16) NUMBITS(0x20) [],
        // Access Flag
        AF OFFSET(10) NUMBITS(0b1) [
            False = 0,
            True = 1
        ],
        // Shareability
        SH OFFSET(8) NUMBITS(0b10) [
            NonShareable = 0b00,
            Unpredictable = 0b01,
            OuterShareable = 0b10,
            InnerShareable = 0b11
        ],
        // Access Permissions
        AP OFFSET(6) NUMBITS(0b10) [
            RW_EL1 = 0b00,
            RW_EL0 = 0b01,
            RO_EL1 = 0b10,
            RO_EL0 = 0b11
        ],
        // MemAttr: choose which MAIR index to use
        MemAttr OFFSET(2) NUMBITS(0b11) [],
        // Is this the end of the translation table walk or not?
        TYPE OFFSET(1) NUMBITS(0b1) [
            Block = 0b0,
            Table = 0b1
        ],
        VALID OFFSET(0) NUMBITS(0b1) [
            False = 0b0,
            True = 0b1
        ]
    ]
}
// definitely not doing this bit right, figure it out later
#[repr(C)]
#[repr(align(65536))]
struct KernTranslationTables {
    l1: [InMemoryRegister<u64, TABLE_DESC::Register>; 64],
    l2: [InMemoryRegister<u64, TABLE_DESC::Register>; 8192],
    l3: [InMemoryRegister<u64, TABLE_DESC::Register>; 8192],

}
#[inline(never)]
pub unsafe fn init() {
    // Layout we use here:
    // 64 KiB pages, 3 levels (48 bit address space, 256 TiB address space split between lower and higher half)
    // I'll have to make a 16 KiB, 3 levels 47? bit address space later for Apple boards


    // Initialize the Translation Control Register
    TCR_EL1.write(
        TCR_EL1::TG1::KiB_64 // 64 KiB granule size for kernel space
        + TCR_EL1::IPS::Bits_48 // 48 bit addresses (256TiB address space)
        + TCR_EL1::T1SZ.val(17) // 2^(64-17) = 2^47 which is half of 2^48
        + TCR_EL1::TG0::KiB_64 // 64 KiB granule size for user space
        + TCR_EL1::T0SZ.val(17) // ditto for T1SZ lul
        + TCR_EL1::TBI0::Ignored // Ignore top byte of VA
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Outer Cacheability attribute
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Inner Cacheability attribute
        + TCR_EL1::EPD0::EnableTTBR0Walks // self explanatory
        + TCR_EL1::EPD1::EnableTTBR1Walks // self explanatory
        + TCR_EL1::SH1::Inner // inner shareability for TTBR1
        + TCR_EL1::SH0::Inner // outer shareability for TTBR0
    );
    // Initialize Memory Attributes
    MAIR_EL1.write(
        MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc // Outer DRAM Cacheability
        + MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc // Ditto for Outer
        + MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck // Device memory for MMIO
    );
    // Idenitity map 4GB of memory for lower half

    // Map kernel base to 0xFFFF0000_00000000 for upper half

    // Actually enable the MMU. Literally the next instruction fetch
    // will use the translation tables, so be careful
    // Just comment out the whole MMU init call from kinit so it doesn't die
    SCTLR_EL1.modify(
        SCTLR_EL1::M::Enable // Enable the MMU
        + SCTLR_EL1::C::Cacheable // Enable data cache
        + SCTLR_EL1::I::Cacheable // Enable instruction cache
    );
    // required memory fence because ordering
    barrier::isb(barrier::SY);
}
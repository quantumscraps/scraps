use cortex_a::{barrier, regs::*};

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
        + MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck
    );
    
}
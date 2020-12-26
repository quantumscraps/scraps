use cortex_a::{barrier, regs::*};
use register::{mmio::*, register_bitfields, InMemoryRegister};
use core::convert;

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

#[inline(never)]
pub unsafe fn init() {
    /* * * * * * * * * * * * * * * * * * * * * * * * * * * *
     * 1) Initialize TCR_EL1 (done)                        *
     * 2) Initialize MAIR_EL1 (easy)                       *
     * 3) Build up translation tables and set TTBR (hard)  *
     * 4) Enable MMU (easy)                                *
     * * * * * * * * * * * * * * * * * * * * * * * * * * * */
    TCR_EL1.write(
        TCR_EL1::TG1::KiB_64       // 64 KiB granule for higher half
        + TCR_EL1::TG0::KiB_64     // 64 KiB granule for lower half
        + TCR_EL1::IPS.val(        // Maximum supported Physical Address range
            ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange)
        )
        + TCR_EL1::AS::ASID16Bits  // 16 bit address space ID
        + TCR_EL1::T0SZ.val(16)    // 48 bit VA in lower half (64-48=16)
        + TCR_EL1::T1SZ.val(16)    // 48 bit VA in upper half (64-48=16)
        + TCR_EL1::TBI0::Used      // Disable lower half VA tagging for now
        + TCR_EL1::TBI1::Used      // Disable higher half VA tagging for now
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Write back inner cacheable (lower)
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Write back outer cacheable (lower)
        + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Write back inner cacheable (higher)
        + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Write back outer cacheable (higher)
        + TCR_EL1::EPD0::EnableTTBR0Walks // Self explanatory
        + TCR_EL1::EPD1::EnableTTBR1Walks // Self explanatory
        + TCR_EL1::SH0::Inner      // Inner shareability lower half
        + TCR_EL1::SH1::Inner      // Inner shareability upper half
    );
}
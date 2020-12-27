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
        // Is this a big block or not?
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
#[derive(Copy, Clone)]
#[repr(transparent)]
struct TableDescriptor(u64);

#[repr(C)]
#[repr(align(65536))]
struct TempTranslationTables {
    l3: [[TableDescriptor; 8192]; 8],
    l2: [TableDescriptor; 8],
}
impl TempTranslationTables {
    const fn new() -> Self {
        Self {
            l3: [[TableDescriptor(0); 8192]; 8],
            l2: [TableDescriptor(0); 8]
        }
    }
}
static mut TEMP_TABLES: TempTranslationTables = TempTranslationTables::new();
trait EzRef {
    fn get_addr(&self) -> usize;
}
impl<T, const N: usize> EzRef for [T; N] {
    fn get_addr(&self) -> usize {
        self as *const _ as usize
    }
}
impl convert::From<usize> for TableDescriptor {
    fn from(addr: usize) -> Self {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(0);
        temp.write(
            TABLE_DESC::ADDR.val(addr as u64 >> 16)
            + TABLE_DESC::TYPE::Table
            + TABLE_DESC::VALID::True
        );
        Self(temp.get())
    }
}
impl TableDescriptor {
    fn new(addr: usize) -> Self {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(0);
        temp.write(
            TABLE_DESC::ADDR.val(addr as u64 >> 16)
            + TABLE_DESC::AF::True
            + TABLE_DESC::TYPE::Table
            + TABLE_DESC::VALID::True
            + TABLE_DESC::SH::OuterShareable
            + TABLE_DESC::MemAttr.val(1)
        );
        Self(temp.get())
    }
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
        + TCR_EL1::T0SZ.val(22)    // 42 bit VA in lower half (64-42=22)
        + TCR_EL1::T1SZ.val(22)    // 42 bit VA in upper half (64-42=22)
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

    // Define 0 as normal cacheable DRAM and 1 as device/mmio
    MAIR_EL1.write(
        MAIR_EL1::Attr0_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc   // Outer DRAM Cacheability
        + MAIR_EL1::Attr0_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc // Ditto for inner
        + MAIR_EL1::Attr1_Device::nonGathering_nonReordering_EarlyWriteAck    // MMIO
    );

    // For now just map 4 GiB address space as non cacheable using 512MiB blocks
    // This will be refactored later
    /*for i in 0..8 {
        TEMP_TABLES.l2[0] = (i * 512 * 1024 * 1024 | 0b01 | (1 << 10) | (2 << 8) | (0b001 << 2) | (0 << 7) | (1 << 53));
    }
    TTBR0_EL1.set_baddr(TEMP_TABLES.l2.get_addr() as u64);
    TTBR0_EL1.modify(TTBR0_EL1::CnP::SET);
    TTBR1_EL1.set_baddr(TEMP_TABLES.l2.get_addr() as u64);
    TTBR1_EL1.modify(TTBR1_EL1::CnP::SET);
    */
    for (i, l2_thing) in TEMP_TABLES.l2.iter_mut().enumerate() {
        *l2_thing = TEMP_TABLES.l3[i].get_addr().into();

        for (j, l3_entry) in TEMP_TABLES.l3[i].iter_mut().enumerate() {
            let result = i << 29 + j << 16;
            *l3_entry = TableDescriptor::new(result);
        }
    }
    TTBR0_EL1.set_baddr(TEMP_TABLES.l2.get_addr() as u64);
    // Enable MMU
    barrier::isb(barrier::SY);
    SCTLR_EL1.modify(
        SCTLR_EL1::M::Enable // Enable the MMU
        + SCTLR_EL1::C::Cacheable // Enable data cache
        + SCTLR_EL1::I::Cacheable // Enable instruction cache
    );
    barrier::isb(barrier::SY);
}
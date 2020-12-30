use core::convert;
use cortex_a::{barrier, regs::*};
use crate::{print, println};
use register::{register_bitfields, InMemoryRegister};
use crate::physical_page_allocator::{ALLOCATOR, PAGE_SIZE};


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
pub struct TableDescriptor(u64);

#[repr(C)]
#[repr(align(65536))]
pub struct Table {
    pub entries: [TableDescriptor; 8192]
}
impl TableDescriptor {
    pub fn is_valid(&self) -> bool {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(self.0);
        println!("is_valid: {}", temp.is_set(TABLE_DESC::VALID));
        temp.is_set(TABLE_DESC::VALID)
    }
}
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
                + TABLE_DESC::VALID::True,
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
                + TABLE_DESC::TYPE::Table // Make this changeable later
                + TABLE_DESC::VALID::True
                + TABLE_DESC::SH::OuterShareable
                + TABLE_DESC::MemAttr.val(1),
        );
        Self(temp.get())
    }
    fn get_addr(&self) -> usize {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(self.0);
        temp.read(TABLE_DESC::ADDR) as usize
    }
}

// This one is for 64K page size 2 level 42 bit vaddr
pub fn map_one_page(root: &mut Table, virt_addr: usize, phys_addr: usize, level: usize) {
    // Split virtual address into components
    let virt_indexes = [
        (virt_addr >> 29) & 0x1fff, // level 2 (512M)
        (virt_addr >> 16) & 0x1fff  // level 3 (64K)
    ];
    println!("virt_indexes({:#x}): {:#x} {:#x}", virt_addr, virt_indexes[0], virt_indexes[1]);
    // First component
    let mut v = &mut root.entries[virt_indexes[0]];
    // Iterate over levels till we get to the right level
    // v will store the correct PTE location at the end
    // level = 0 -> 64K page, level = 1 -> 512M block
    for i in level..1 {
        // if the index in the entry is not valid
        if !v.is_valid() {
            // get us a page and mark it as valid in the table for the next level
            let page = unsafe { ALLOCATOR.try_allocate(PAGE_SIZE).expect("Couldn't allocate page") };
            *v = unsafe { (page as usize).into() };
        }
        // now, get the new table descriptor pointing to the PTE
        // and advance v past it
        let entry: *mut TableDescriptor = v;
        v = unsafe {entry.add(virt_indexes[i]).as_mut().unwrap() };
    }
    if !v.is_valid() {
        // get us a page and mark it as valid in the table for the next level
        let page = unsafe { ALLOCATOR.try_allocate(PAGE_SIZE).expect("Couldn't allocate page") };
        *v = unsafe { (page as usize).into() };
    }
    // Now that v contains the right PTE pointer, store the right PTE!!!!
    *v = TableDescriptor::new(phys_addr);
}
#[inline(never)]
/// # Safety
/// Only safe to call once.
pub unsafe fn init() {
    /* * * * * * * * * * * * * * * * * * * * * * * * * * * *
     * 1) Initialize TCR_EL1 (done)                        *
     * 2) Initialize MAIR_EL1 (easy)                       *
     * 3) Build up translation tables and set TTBR (hard)  *
     * 4) Enable MMU (easy)                                *
     * * * * * * * * * * * * * * * * * * * * * * * * * * * */
    TCR_EL1.write(
        TCR_EL1::TG0::KiB_64       // 64 KiB granule for higher half
        //+ TCR_EL1::TG1::KiB_64     // 64 KiB granule for lower half
        + TCR_EL1::IPS.val(        // Maximum supported Physical Address range
            ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange)
        )
        + TCR_EL1::AS::ASID16Bits  // 16 bit address space ID
        + TCR_EL1::T0SZ.val(64-42) // 39 bit VA in lower half
        //+ TCR_EL1::T1SZ.val(64-42) // 39 bit VA in upper half
        + TCR_EL1::TBI0::Used      // Disable lower half VA tagging for now
        //+ TCR_EL1::TBI1::Used      // Disable higher half VA tagging for now
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Write back inner cacheable (lower)
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Write back outer cacheable (lower)
        //+ TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Write back inner cacheable (higher)
        //+ TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Write back outer cacheable (higher)
        + TCR_EL1::EPD0::EnableTTBR0Walks // Self explanatory
        //+ TCR_EL1::EPD1::EnableTTBR1Walks // Self explanatory
        + TCR_EL1::SH0::Inner      // Inner shareability lower half
        //+ TCR_EL1::SH1::Inner, // Inner shareability upper half
    );
    // Define 0 as normal cacheable DRAM and 1 as device/mmio
    MAIR_EL1.write(
        MAIR_EL1::Attr0_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc   // Outer DRAM Cacheability
        + MAIR_EL1::Attr0_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc // Ditto for inner
        + MAIR_EL1::Attr1_Device::nonGathering_nonReordering_EarlyWriteAck, // MMIO
    );
    // create the root PTE
    let mut root_table_u8 = unsafe { ALLOCATOR.try_allocate(PAGE_SIZE).expect("Couldn't allocate page") };
    let mut table = core::mem::transmute::<&mut u8, &mut Table>(&mut unsafe { *root_table_u8 });
    // identity map 8 pages starting from just before the kernel and mmio
    for i in 0..8 {
        map_one_page(table, 0x80000-PAGE_SIZE + i*PAGE_SIZE, 0x80000-PAGE_SIZE + i*PAGE_SIZE, 0);
    }
    for i in 0..8 {
        map_one_page(table, crate::bsp::mmio_base() + i*PAGE_SIZE, crate::bsp::mmio_base() + i*PAGE_SIZE, 0);
    }
    // Set up the TTBR0_EL1
    TTBR0_EL1.set_baddr(root_table_u8 as *const _ as u64);
    // Enable MMU
    barrier::isb(barrier::SY);
    SCTLR_EL1.modify(
        SCTLR_EL1::M::Enable       // Enable the MMU
        + SCTLR_EL1::C::NonCacheable  // Enable data cache
        + SCTLR_EL1::I::NonCacheable, // Enable instruction cache
    );
    barrier::isb(barrier::SY);
}

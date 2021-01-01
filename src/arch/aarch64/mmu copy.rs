/*
use core::convert;
use cortex_a::{barrier, regs::*};
use crate::{print, println};
use register::{register_bitfields, InMemoryRegister};
use crate::physical_page_allocator::{ALLOCATOR, PAGE_SIZE};
mod level {
    pub const KiB_64: usize = 0;
    pub const MiB_512: usize = 1;
    pub const TiB_4: usize = 2;
}
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
    fn set_page(&mut self, addr: usize) {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(self.0);
        temp.write(
            TABLE_DESC::ADDR.val(addr as u64 >> 16)
                + TABLE_DESC::AF::True
                + TABLE_DESC::TYPE::Block // Make this changeable later
                + TABLE_DESC::VALID::True
                + TABLE_DESC::SH::OuterShareable
                + TABLE_DESC::MemAttr.val(0),
        );
        self.0 = temp.get();
    }
    fn get_addr(&self) -> usize {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(self.0);
        (temp.read(TABLE_DESC::ADDR) << 16) as usize
    }
    pub fn is_valid(&self) -> bool {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(self.0);
        println!("is_valid: {}", temp.is_set(TABLE_DESC::VALID));
        temp.is_set(TABLE_DESC::VALID)
    }
    pub fn is_block(&self) -> bool {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(self.0);
        !temp.is_set(TABLE_DESC::TYPE)
    }
    pub fn get_child(&self) -> *mut TableDescriptor {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(self.0);
        (temp.read(TABLE_DESC::ADDR) << 16) as *mut TableDescriptor
    }
    pub fn set_child(&mut self, addr: usize)  {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(self.0);
        temp.write(
            TABLE_DESC::ADDR.val(addr as u64 >> 16)
                + TABLE_DESC::TYPE::Table
                + TABLE_DESC::VALID::True,
        );
        self.0 = temp.get();
    }
}
pub fn map_page(root: &mut Table, vaddr: usize, paddr: usize, level: usize) {
    let indexes = [ // address components
        (vaddr >> 42) & 0x3f,   // level 1 (4   TiB)
        (vaddr >> 29) & 0x1fff, // level 2 (512 MiB)
        (vaddr >> 16) & 0x1fff, // level 3 (64  KiB)
    ];
    println!("indexes({:#x}): {:#x} {:#x} {:#x}", vaddr, indexes[0], indexes[1], indexes[2]);
    // get the right PTE location in the root table
    let mut v = &mut root.entries[indexes[0]];
    // now, for each level we need to make sure the pointer to the next level
    // table is a valid one, then in that one we need to 
    // levels: 
    for i in level..2 {
        if !v.is_valid() {
            // if CHILD page table isn't valid get a new one and mark it as valid
            let new_page = unsafe { ALLOCATOR.try_zallocate(PAGE_SIZE).expect("Couldn't allocate page") };
            v.set_child(new_page as usize);
        }
        // go to next level table pointed to by the selected PTE
        // child contains the top of the child page table
        // now offset the pointer to get to the right entry
        v = unsafe { v.get_child().add(indexes[i+1]).as_mut().unwrap() };
    }
    // now that we have the final page table entry, set the page address
    v.set_page(paddr);
}
pub fn virt_to_phys(root: &Table, vaddr: usize) -> Option<usize> {
    let indexes = [ // address components
        (vaddr >> 42) & 0x3f,   // level 1 (4   TiB)
        (vaddr >> 29) & 0x1fff, // level 2 (512 MiB)
        (vaddr >> 16) & 0x1fff, // level 3 (64  KiB)
    ];
    let mut v = &root.entries[indexes[0]];
    for i in (0..=2) {
        if !v.is_valid() {
            println!("V not valid @ i={}", i);
            break;
        } else if v.is_block() {
            return Some(v.get_addr() + (vaddr & 0xFFFF));
        }
        v = unsafe { v.get_child().add(indexes[i+1]).as_mut().unwrap() };
    }
    None
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
        + TCR_EL1::TG1::KiB_64     // 64 KiB granule for lower half
        + TCR_EL1::IPS.val(        // Maximum supported Physical Address range
            ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange)
        )
        //+ TCR_EL1::AS::ASID16Bits  // 16 bit address space ID
        + TCR_EL1::T0SZ.val(64-42) // 39 bit VA in lower half
        + TCR_EL1::T1SZ.val(64-42) // 39 bit VA in upper half
        + TCR_EL1::TBI0::Used      // Disable lower half VA tagging for now
        + TCR_EL1::TBI1::Used      // Disable higher half VA tagging for now
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Write back inner cacheable (lower)
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Write back outer cacheable (lower)
        + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Write back inner cacheable (higher)
        + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable // Write back outer cacheable (higher)
        + TCR_EL1::EPD0::EnableTTBR0Walks // Self explanatory
        + TCR_EL1::EPD1::EnableTTBR1Walks // Self explanatory
        + TCR_EL1::SH0::Inner      // Inner shareability lower half
        + TCR_EL1::SH1::Inner, // Inner shareability upper half
    );
    // Define 0 as normal cacheable DRAM and 1 as device/mmio
    MAIR_EL1.write(
        MAIR_EL1::Attr0_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc   // Outer DRAM Cacheability
        + MAIR_EL1::Attr0_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc // Ditto for inner
        + MAIR_EL1::Attr1_Device::nonGathering_nonReordering_EarlyWriteAck, // MMIO
    );
    // create the root PTE
    let mut root_table_u8 = unsafe { ALLOCATOR.try_zallocate(PAGE_SIZE).expect("Couldn't allocate page") };
    let mut table = core::mem::transmute::<&mut u8, &mut Table>(&mut unsafe { *root_table_u8 });
    // identity map 8 pages starting from just before the kernel and mmio
    /*for i in 0..8 {
        map_page(table, 0x80000-PAGE_SIZE + i*PAGE_SIZE, 0x80000-PAGE_SIZE + i*PAGE_SIZE, 0);
    }
    for i in 0..8 {
        map_page(table, crate::bsp::mmio_base() + i*PAGE_SIZE, crate::bsp::mmio_base() + i*PAGE_SIZE, 0);
    }*/
    for i in 0..8 {
        map_page(table, i * 512 * 1024 * 1024, i * 512 * 1024 * 1024, level::MiB_512);
    }
    match virt_to_phys(table, 0x69420) {
        Some(addr) => println!("virt_to_phys(0x69420) = {:#x}", addr),
        None => println!("virt_to_phys(0x69420) = PAGE FAULT")
    };
    println!("baddr: {:#x}", root_table_u8 as *const _ as u64);
    // Set up the TTBR0_EL1
    TTBR0_EL1.set_baddr(root_table_u8 as *const _ as u64);
    TTBR1_EL1.set_baddr(root_table_u8 as *const _ as u64);
    // Enable MMU
    barrier::isb(barrier::SY);
    SCTLR_EL1.modify(
        SCTLR_EL1::M::Enable       // Enable the MMU
        + SCTLR_EL1::C::Cacheable  // Enable data cache
        + SCTLR_EL1::I::Cacheable, // Enable instruction cache
    );
    barrier::isb(barrier::SY);
}
*/
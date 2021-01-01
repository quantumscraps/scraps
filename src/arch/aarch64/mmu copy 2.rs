/*
use core::convert;
use cortex_a::{barrier, regs::*};
use crate::{print, println, bsp};
use register::{register_bitfields, InMemoryRegister, FieldValue};
use crate::physical_page_allocator::{ALLOCATOR, PAGE_SIZE};
mod level {
    pub const KiB_4: usize = 0;
    pub const MiB_2: usize = 1;
    pub const GiB_1: usize = 2;
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
        //Actual VA (36 bits not including the 12 bit in page offset)
        ADDR OFFSET(12) NUMBITS(36) [],
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
pub struct Table {
    pub entries: [TableDescriptor; 512]
}

impl TableDescriptor {
    
    fn set_page(&mut self, addr: usize, perms: FieldValue<u64, TABLE_DESC::Register>) {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(self.0);
        temp.write(
            TABLE_DESC::ADDR.val(addr as u64 >> 12)
                + TABLE_DESC::AF::True
                + TABLE_DESC::TYPE::Block // Make this changeable later
                + TABLE_DESC::VALID::True
                + perms
        );
        self.0 = temp.get();
    }
    fn get_addr(&self) -> usize {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(self.0);
        (temp.read(TABLE_DESC::ADDR) << 12) as usize
    }
    pub fn is_valid(&self) -> bool {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(self.0);
        println!("is_valid: {}", temp.is_set(TABLE_DESC::VALID));
        temp.matches_any(TABLE_DESC::VALID::True)
    }
    pub fn is_block(&self) -> bool {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(self.0);
        temp.matches_any(TABLE_DESC::TYPE::Block)
    }
    pub fn get_child(&self) -> *mut TableDescriptor {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(self.0);
        (temp.read(TABLE_DESC::ADDR) << 12) as *mut TableDescriptor
    }
    pub fn set_child(&mut self, addr: usize)  {
        let temp: InMemoryRegister<u64, TABLE_DESC::Register> = InMemoryRegister::new(self.0);
        temp.write(
            TABLE_DESC::ADDR.val(addr as u64 >> 12)
                + TABLE_DESC::TYPE::Table
                + TABLE_DESC::VALID::True,
        );
        self.0 = temp.get();
    }
    pub fn get(&self) -> u64 {
        self.0
    }
}
pub fn map_page(root: &mut Table, vaddr: usize, paddr: usize, level: usize, perms: FieldValue<u64, TABLE_DESC::Register>) {
    let indexes = [ // address components
        (vaddr >> 30) & 0x1ff, // level 1 (1 GiB)
        (vaddr >> 21) & 0x1ff, // level 2 (2 MiB)
        (vaddr >> 12) & 0x1ff, // level 3 (4 KiB)
    ];
    println!("indexes({:#x}): {:#x} {:#x} {:#x}", vaddr, indexes[0], indexes[1], indexes[2]);
    // get the right PTE location in the root table
    let mut v = &mut root.entries[indexes[0]]; // PTE for the gigapage
    // now, for each level we need to make sure the pointer to the next level
    // table is a valid one, then in that one we need to 
    // levels: 
    println!("ogchildaddr: {:#x}", v.get_addr());
    for i in level..2 {
        if !v.is_valid() {
            // if CHILD page table isn't valid get a new one and mark it as valid
            let new_page = unsafe { ALLOCATOR.try_zallocate(PAGE_SIZE).expect("Couldn't allocate page") };
            v.set_child(new_page as usize);
        }
        println!("intchildaddr: {:#x}", v.get_addr());
        // go to next level table pointed to by the selected PTE
        // child contains the top of the child page table
        // now offset the pointer to get to the right entry
        v = unsafe { v.get_child().add(indexes[i+1]).as_mut().unwrap() };
        println!("aftchildaddr: {:#x}", v.get_addr());
    }
    // now that we have the final page table entry, set the page address
    v.set_page(paddr, perms);
    println!("finalchild: {:#x}", v.get_addr());
}
pub fn virt_to_phys(root: &Table, vaddr: usize) -> Option<usize> {
    let indexes = [ // address components
        (vaddr >> 30) & 0x1ff, // level 1 (1 GiB)
        (vaddr >> 21) & 0x1ff, // level 2 (2 MiB)
        (vaddr >> 12) & 0x1ff, // level 3 (4 KiB)
    ];
    let mut v = &root.entries[indexes[0]];
    println!("vtopogchildaddr: {:#x}", v.get_addr());
    for i in 0..=2 {
        if !v.is_valid() {
            println!("V not valid @ i={}", i);
            println!("v: {:#x}: {:#x}", v as *const _ as usize, v.get());
            break;
        } else if v.is_block() {
            println!("yesv: {:#x}: {:#x}", v as *const _ as usize, v.get_addr());
            return Some(v.get_addr() + (vaddr & 0xFFF));
        }
        v = unsafe { v.get_child().add(indexes[i+1]).as_mut().unwrap() };
    }
    None
}

#[inline(never)]
/// # Safety
/// Only safe to call once.
pub unsafe fn init() {
    // Attr0 -> Normal, Attr1 -> device
    MAIR_EL1.write(
        MAIR_EL1::Attr0_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
        + MAIR_EL1::Attr0_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
        + MAIR_EL1::Attr1_Device::nonGathering_nonReordering_EarlyWriteAck
    );
    let mut root_table_0_u8 = unsafe { ALLOCATOR.try_zallocate(PAGE_SIZE).expect("Couldn't allocate page") };
    let mut table_0 = core::mem::transmute::<&mut u8, &mut Table>(&mut *root_table_0_u8);
    for i in 0..2048 {
        if i == 504 {
            continue;
        }
        let perms = match i >= (bsp::mmio_base() >> 21) {
            true => TABLE_DESC::SH::OuterShareable + TABLE_DESC::MemAttr.val(1),
            false => TABLE_DESC::SH::InnerShareable + TABLE_DESC::MemAttr.val(0)
        };
        // map 2M blocks and allow everything
        map_page(table_0, i << 21, i << 21, 1, perms);
    }
    TTBR0_EL1.set_baddr(root_table_0_u8 as *const _ as u64);
    TTBR0_EL1.modify(TTBR0_EL1::CnP::SET);
    TCR_EL1.write(
        TCR_EL1::TBI0::Ignored
        + TCR_EL1::IPS.val(
            ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange)
        )
        + TCR_EL1::TG0::KiB_4
        + TCR_EL1::SH0::Inner
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::EPD0::EnableTTBR0Walks
        + TCR_EL1::T0SZ.val(64-39)
    );
    map_page(table_0, 0x3f00_0000, 0x3f00_0000, 1, TABLE_DESC::SH::InnerShareable + TABLE_DESC::MemAttr.val(0));
    barrier::isb(barrier::SY);
    //SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::NonCacheable + SCTLR_EL1::I::NonCacheable);
    barrier::isb(barrier::SY);
    match virt_to_phys(table_0, 0x3f00_dead) {
        Some(thing) => println!("virt_to_phys(0x3f00_dead) -> {:#x}", thing),
        None => println!("virt_to_phys(0x3f00_dead) -> PAGE FAULT")
    }
    loop {}
}
/*
pub unsafe fn _init() {
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
    TCR_EL1.write()
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
*/
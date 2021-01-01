use core::convert;
use cortex_a::{barrier, regs::*};
use crate::{print, println, bsp};
use modular_bitfield::prelude::*;
use crate::physical_page_allocator::{ALLOCATOR, PAGE_SIZE};
mod level {
    pub const KiB_4: usize = 0;
    pub const MiB_2: usize = 1;
    pub const GiB_1: usize = 2;
}
#[derive(BitfieldSpecifier, Clone, Copy, PartialEq, Eq)]
#[bits = 1]
enum PTEType {
    Block = 0b0,
    Table = 0b1
}
#[derive(BitfieldSpecifier, Clone, Copy, PartialEq, Eq)]
#[bits = 2]
enum Armv8SH {
    NonShareable = 0b00,
    Unpredictable = 0b01,
    OuterShareable = 0b10,
    InnerShareable = 0b11
}
#[derive(BitfieldSpecifier, Clone, Copy, PartialEq, Eq)]
#[bits = 2]
enum Armv8AP {
    RW_EL1 = 0b00,
    RW_EL0 = 0b01,
    RO_EL1 = 0b10,
    RO_EL0 = 0b11
}
#[bitfield]
#[repr(u64)]
#[derive(Default, Clone, Copy)]
pub struct PTE {
    valid: bool,    // Valid [0]
    ptype: PTEType, // Type [1]
    MemAttr: B3,   // MemAttr [2:4]
    ns: bool,      // NS [5]
    ap: Armv8AP,   // AP [6:7]
    sh: Armv8SH,   // SH [8:9]
    af: bool,      // AF [10]
    nG: bool,      // nG [11]
    addr: B36,     // addr [12:47]
    _res0: B4,     // res [48:51]
    c: bool,       // C [52]
    pxn: bool,     // PXN [53]
    uxn: bool,     // UXN [54]
    _res1: B9      // res [55:63]
}

impl PTE {
    fn from_addr(addr: usize) -> Self {
        Self::new().with_addr((addr as u64) >> 12)
    }
    fn phys_addr(&self) -> usize {
        (self.addr() as usize) << 12 
    }
    fn is_invalid(&self) -> bool {
        !self.valid()
    }
    fn get_val(&self) -> u64 {
        let b = self.into_bytes();
        let mut a: u64 = 0;
        for i in 0..8 {
            a |= (b[i] as u64) << (8 * i);
        }
        a
    }
}
#[repr(C)]
#[repr(align(4096))]
pub struct PageTable {
    entries: [PTE; 512]
}

pub fn map_page(root: &mut PageTable, vaddr: usize, paddr: usize, level: usize) {
    let indexes = [
        (vaddr >> 30) & 0x1FF,
        (vaddr >> 21) & 0x1FF,
        (vaddr >> 12) & 0x1FF
    ];
    // now pte_ptr has the l1 entry pointing to the l2 table
    let mut pte_ptr = &mut root.entries[indexes[0]];
    for i in 0..(2 - level) {
        if pte_ptr.is_invalid() {
            println!("allocating new table...");
            let next_level_table = unsafe { ALLOCATOR.try_zallocate(PAGE_SIZE).expect("Couldn't allocate page") };
            pte_ptr.set_addr((next_level_table as u64) >> 12);
            pte_ptr.set_ptype(PTEType::Table);
            pte_ptr.set_af(true);
            pte_ptr.set_valid(true);
            println!("ptval: {:#x}", pte_ptr.get_val());
        }
        pte_ptr = unsafe { (pte_ptr.phys_addr() as *mut PTE).add(indexes[i+1]).as_mut().unwrap() };
    }
    pte_ptr.set_ptype(PTEType::Block);
    pte_ptr.set_valid(true);
    pte_ptr.set_af(true);
    pte_ptr.set_addr((paddr as u64 >> 12) & 0xFF_FFFF_FFFF);
}

pub fn virt_to_phys(root: &PageTable, vaddr: usize) -> Option<usize> {
    let indexes = [ // address components
        (vaddr >> 30) & 0x1ff, // level 1 (1 GiB)
        (vaddr >> 21) & 0x1ff, // level 2 (2 MiB)
        (vaddr >> 12) & 0x1ff, // level 3 (4 KiB)
    ];
    let mut v = &root.entries[indexes[0]];
    for i in 0..=2 {
        if v.is_invalid() {
            println!("V not valid @ i={}", i);
            break;
        } else if v.ptype() == PTEType::Block {
            // i = 2: 4k, i = 1: 2m, i = 0: 1g
            return Some(v.phys_addr() + (vaddr & match i {
                0 => 0x3fffffff,
                1 => 0x1fffff,
                _ => 0xfff
            }));
        }
        v = unsafe { (v.phys_addr() as *mut PTE).add(indexes[i+1]).as_mut().unwrap() };
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
    let mut table_0 = core::mem::transmute::<&mut u8, &mut PageTable>(&mut *root_table_0_u8);
    /*for i in 0..2048 {
        if i == 504 {
            continue;
        }
        let perms = match i >= (bsp::mmio_base() >> 21) {
            true => TABLE_DESC::SH::OuterShareable + TABLE_DESC::MemAttr.val(1),
            false => TABLE_DESC::SH::InnerShareable + TABLE_DESC::MemAttr.val(0)
        };
        // map 2M blocks and allow everything
        map_page(table_0, i << 21, i << 21, 1, perms);
    }*/
    for i in 0..2048 {
        map_page(table_0, i << 21, i << 21, 1);
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
    barrier::isb(barrier::SY);
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::NonCacheable + SCTLR_EL1::I::NonCacheable);
    barrier::isb(barrier::SY);
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
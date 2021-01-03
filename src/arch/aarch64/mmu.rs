use cortex_a::{barrier, regs::*};
use modular_bitfield::prelude::*;
use crate::physical_page_allocator::{ALLOCATOR, PAGE_SIZE};

mod level {
    //pub const GiB_1: usize = 0;
    pub const MiB_2: usize = 1;
    //pub const KiB_4: usize = 2;
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
    RwEl1 = 0b00,
    RwEl0 = 0b01,
    RoEl1 = 0b10,
    RoEl0 = 0b11
}

#[bitfield]
#[repr(u64)]
#[derive(Default, Clone, Copy)]
pub struct PTE {
    valid: bool,    // Valid [0]
    ptype: PTEType, // Type [1]
    mem_attr: B3,   // mem_attr [2:4]
    ns: bool,      // NS [5]
    ap: Armv8AP,   // AP [6:7]
    sh: Armv8SH,   // SH [8:9]
    af: bool,      // AF [10]
    ng: bool,      // ng [11]
    addr: B36,     // addr [12:47]
    #[skip]
    res0: B4,     // res [48:51]
    c: bool,       // C [52]
    pxn: bool,     // PXN [53]
    uxn: bool,     // UXN [54]
    #[skip]
    res1: B9      // res [55:63]
}

impl PTE {
    /*fn from_addr(addr: usize) -> Self {
        Self::new().with_addr((addr as u64) >> 12)
    }*/
    fn phys_addr(&self) -> usize {
        (self.addr() as usize) << 12 
    }
    fn is_invalid(&self) -> bool {
        !self.valid()
    }
    /*fn get_val(&self) -> u64 {
        let b = self.into_bytes();
        let mut a: u64 = 0;
        for i in 0..8 {
            a |= (b[i] as u64) << (8 * i);
        }
        a
    }*/
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
    for i in 0..level {
        if pte_ptr.is_invalid() {
            let next_level_table = unsafe { ALLOCATOR.try_zallocate(PAGE_SIZE).expect("Couldn't allocate page") };
            pte_ptr.set_addr((next_level_table as u64) >> 12);
            pte_ptr.set_ptype(PTEType::Table);
            pte_ptr.set_af(true);
            pte_ptr.set_valid(true);
        }
        pte_ptr = unsafe { (pte_ptr.phys_addr() as *mut PTE).add(indexes[i+1]).as_mut().unwrap() };
    }
    pte_ptr.set_ptype(PTEType::Block);
    pte_ptr.set_valid(true);
    pte_ptr.set_af(true);
    if paddr < 0xFE00_0000 {
        pte_ptr.set_mem_attr(1);
        pte_ptr.set_sh(Armv8SH::InnerShareable);
    } else {
        pte_ptr.set_mem_attr(0);
        pte_ptr.set_sh(Armv8SH::OuterShareable);
    }
    pte_ptr.set_addr((paddr as u64 >> 12) & 0xF_FFFF_FFFF);
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
        MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
        + MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
        + MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck
    );
    let root_table_0_u8 = ALLOCATOR.try_zallocate(PAGE_SIZE).expect("Couldn't allocate page");
    let table_0 = core::mem::transmute::<&mut u8, &mut PageTable>(&mut *root_table_0_u8);
    for i in 0..2048 {
        map_page(table_0, i << 21, i << 21, level::MiB_2);
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
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    barrier::isb(barrier::SY);
}
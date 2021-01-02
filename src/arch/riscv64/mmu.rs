#![allow(dead_code)]

use modular_bitfield::prelude::*;

use crate::{
    physical_page_allocator::{ALLOCATOR, PAGE_SIZE},
    print, printk,
};

#[bitfield]
#[repr(u64)]
#[derive(Default, Clone, Copy)]
pub struct Sv39PTE {
    valid: bool,
    #[bits = 3]
    permissions: XWRPermissions,
    user: bool,
    global: bool,
    accessed: bool,
    dirty: bool,
    reserved_for_software: B2,
    ppn0: B9,
    ppn1: B9,
    ppn2: B26,
    reserved: B10,
}
// #[bitfield]
// #[repr(u64)]
// #[derive(Default, Clone, Copy)]
// pub struct Sv39PTE {
//     reserved: B10,
//     ppn2: B26,
//     ppn1: B9,
//     ppn0: B9,
//     reserved_for_software: B2,
//     dirty: bool,
//     accessed: bool,
//     global: bool,
//     user: bool,
//     #[bits = 3]
//     permissions: XWRPermissions,

//     valid: bool,
// }

impl Sv39PTE {
    fn from_addr(addr: u64) -> Self {
        let (ppn2, ppn1, ppn0) = split_phys_addr(addr);
        Self::new().with_ppn2(ppn2).with_ppn1(ppn1).with_ppn0(ppn0)
    }

    fn phys_addr(&self) -> u64 {
        unsplit_phys_addr((self.ppn2(), self.ppn1(), self.ppn0()))
    }
}

/// Splits a Sv39 virtual address into (VPN[2], VPN[1], VPN[0])
fn split_virt_addr_sv39(addr: u64) -> (u16, u16, u16) {
    let vpn0 = (addr >> 12) & ((1 << 9) - 1);
    let vpn1 = (addr >> 21) & ((1 << 9) - 1);
    let vpn2 = (addr >> 30) & ((1 << 9) - 1);
    (vpn2 as u16, vpn1 as u16, vpn0 as u16)
}

/// Splits a physical address into (PPN[2], PPN[1], PPN[0])
fn split_phys_addr(addr: u64) -> (u32, u16, u16) {
    let ppn0 = (addr >> 12) & ((1 << 9) - 1);
    let ppn1 = (addr >> 21) & ((1 << 9) - 1);
    let ppn2 = (addr >> 30) & ((1 << 26) - 1);
    // let ppn2 = (addr >> 12) & ((1 << 26) - 1);
    // let ppn1 = (addr >> 38) & ((1 << 9) - 1);
    // let ppn0 = (addr >> 47) & ((1 << 9) - 1);
    // printk!(
    //     "Split {:056x} -> {:026x}, {:09x}, {:09x}",
    //     addr,
    //     ppn2,
    //     ppn1,
    //     ppn0
    // );
    (ppn2 as u32, ppn1 as u16, ppn0 as u16)
}

/// Merges (PPN[2], PPN[1], PPN[0]) into a physical address.
fn unsplit_phys_addr(parts: (u32, u16, u16)) -> u64 {
    let ppn2 = parts.0 as u64;
    let ppn1 = parts.1 as u64;
    let ppn0 = parts.2 as u64;
    let res = (ppn0 << 12) | (ppn1 << 21) | (ppn2 << 30);
    // printk!(
    //     "Unsplit {:026x}, {:09x}, {:09x} -> {:056x}",
    //     ppn2,
    //     ppn1,
    //     ppn0,
    //     res
    // );
    res
}

/// Divides a constant by the size of a type.
const fn type_divide<T: Sized>(n: usize) -> usize {
    let t_sz = core::mem::size_of::<T>();
    if n % t_sz != 0 {
        panic!("Size of type does not divide N evenly");
    }
    n / t_sz
}

pub struct PageTable<PTE: Sized>
where
    [PTE; type_divide::<PTE>(PAGE_SIZE)]: Sized,
{
    entries: [PTE; type_divide::<PTE>(PAGE_SIZE)],
}

pub type Sv39PageTable = PageTable<Sv39PTE>;

impl Sv39PageTable {
    pub fn print(&self) {
        self.print_indented(0);
    }

    fn print_indented(&self, indent: usize) {
        for (i, ent) in self.entries.iter().enumerate() {
            for _ in 0..indent {
                print!(" ");
            }
            print!("{:04} ", i);
            if ent.valid() {
                print!("V");
            } else {
                print!(" ");
            }
            if ent.global() {
                print!("G");
            } else {
                print!(" ");
            }
            let perms = ent.permissions();
            if ent.valid() && perms == XWRPermissions::Pointer {
                print!("***");
            } else {
                if (perms as usize) & (XWRPermissions::ReadOnly as usize) > 0 {
                    print!("R");
                } else {
                    print!(" ");
                }
                if (perms as usize) & (XWRPermissions::WriteOnly as usize) > 0 {
                    print!("W");
                } else {
                    print!(" ");
                }
                if (perms as usize) & (XWRPermissions::ExecOnly as usize) > 0 {
                    print!("X");
                } else {
                    print!(" ");
                }
            }
            print!(" -> 0x{:x}\n", ent.phys_addr());
            // If this PTE is a indirection, print out the next level
            if ent.valid() && perms == XWRPermissions::Pointer {
                let next_level_table = unsafe { &*(ent.phys_addr() as *const Sv39PageTable) };
                next_level_table.print_indented(indent + 1);
            }
        }
    }
}

pub const ONEGIG: u64 = 0x40000000;

/// maps a 1g gigapage by rounding the given addr
pub fn map_gigapage(root: &mut Sv39PageTable, virt_addr: u64, phys_addr: u64) {
    // let virt_addr = virt_addr & !(onegig - 1);
    // round the address down
    let index = virt_addr / ONEGIG;
    printk!("Mapping root index {}", index);
    // mask out 1g of phys_addr
    let phys_addr2 = phys_addr & !(ONEGIG - 1);
    // set flags to vgrwx for now
    root.entries[index as usize] = Sv39PTE::from_addr(phys_addr2)
        .with_valid(true)
        .with_global(true)
        .with_permissions(XWRPermissions::ReadWriteExec);
}

/// maps a 4k page by rounding the given addr
pub fn map_page(root: &mut Sv39PageTable, virt_addr: u64, phys_addr: u64) {
    // mask out page_size of phys_addr
    let phys_addr2 = phys_addr & !(PAGE_SIZE as u64 - 1);
    // split virt addr
    let (vpn2, vpn1, vpn0) = split_virt_addr_sv39(virt_addr);
    printk!(
        "root index = {}, level1 index = {}, level2 index = {}",
        vpn2,
        vpn1,
        vpn0
    );
    let root_entry = &mut root.entries[vpn2 as usize];
    // if invalid, allocate page
    // otherwise, use new pointer
    let level1_table = if root_entry.valid() {
        unsafe { &mut *(root_entry.phys_addr() as *mut Sv39PageTable) }
    } else {
        // allocate page
        let new_addr = unsafe { &mut ALLOCATOR }
            .try_zallocate(PAGE_SIZE)
            .expect("Failed to allocate page!") as u64;
        // set entry
        *root_entry = Sv39PTE::from_addr(new_addr)
            .with_valid(true)
            .with_permissions(XWRPermissions::Pointer);
        let table = unsafe { &mut *(new_addr as *mut Sv39PageTable) };
        table
    };
    // same thing for level2
    let level1_entry = &mut level1_table.entries[vpn1 as usize];
    let level2_table = if level1_entry.valid() {
        unsafe { &mut *(level1_entry.phys_addr() as *mut Sv39PageTable) }
    } else {
        // allocate page
        let new_addr = unsafe { &mut ALLOCATOR }
            .try_zallocate(PAGE_SIZE)
            .expect("Failed to allocate page!") as u64;
        // set entry
        *level1_entry = Sv39PTE::from_addr(new_addr)
            .with_valid(true)
            .with_permissions(XWRPermissions::Pointer);
        let table = unsafe { &mut *(new_addr as *mut Sv39PageTable) };
        table
    };
    let level2_entry = &mut level2_table.entries[vpn0 as usize];
    // check that the entry is not valid already
    if level2_entry.valid() {
        panic!("Trying to overwrite a valid PTE entry");
    }
    // set leaf
    *level2_entry = Sv39PTE::from_addr(phys_addr2)
        .with_valid(true)
        .with_global(true)
        .with_permissions(XWRPermissions::ReadWriteExec);
}

/// Maps a range of pages using [map_page]
/// begin address is rounded down, end address is rounded up
pub fn map_page_range(
    root: &mut Sv39PageTable,
    virt_addr_begin: u64,
    virt_addr_end: u64,
    phys_addr_begin: u64,
) {
    let page_size_u64 = PAGE_SIZE as u64;
    let virt_offset = virt_addr_begin / page_size_u64;
    let phys_offset = phys_addr_begin / page_size_u64;
    let virt_addr_begin = virt_addr_begin & !(page_size_u64 - 1);
    let virt_addr_end = if virt_addr_end % page_size_u64 > 0 {
        ((virt_addr_end) & !(page_size_u64 - 1)) + page_size_u64
    } else {
        virt_addr_end
    };
    let len = (virt_addr_end - virt_addr_begin) / page_size_u64;
    printk!("Mapping {} pages", len);
    for i in 0..len {
        let virt_addr = (i + virt_offset) * page_size_u64;
        let phys_addr = (i + phys_offset) * page_size_u64;
        printk!("Mapping {:x} -> {:x}", virt_addr, phys_addr);
        map_page(root, virt_addr, phys_addr);
    }
}

/// Enables S-mode
///
/// # Safety
/// The return address must be pointing to a valid instruction.
pub unsafe fn enable_smode(return_to: usize) {
    printk!("setting mstatus to s-mode, zeroing mie");
    let mstatus_val = 1 << 11; // | (1 << 5);
    asm!(
        "csrw mstatus, {0}",
        in(reg) mstatus_val,
    );
    asm!("csrw mie, zero");
    // printk!("Setting const to ra...");
    // // let mut ra2: u64;
    // // asm!("add {0}, ra, zero", out(reg) ra2);
    // __PAGING_RA = ra2;
    // printk!("ra = {:x}", ra2);
    printk!("setting mepc to part2...");
    asm!(
        "csrw mepc, {0}",
        in(reg) return_to,
    );
    printk!("mret...");
    asm!("mret", options(noreturn));
}

/// Looks up a virtual address with the given root table
/// and returns the physical address
///
/// using the given ptesize and levels
///
/// Directly translated from RISC-V Privileged Specification, Section 4.3.2
pub fn table_lookup(mut table: &Sv39PageTable, virt_addr: u64, levels: usize) -> u64 {
    let vpn = vpn_split_variable(virt_addr, levels);
    // 1. a = table, i = levels - 1
    let mut i = levels - 1;
    while i > 0 {
        // 2. pte = a+va.vpn[i]*PTESIZE
        // PTESIZE is accounted for in entry indexing
        let pte: Sv39PTE = table.entries[vpn[i] as usize];
        // 3. If pte.v = 0 or (pte.r = 0 && pte.w = 1) raise fault
        if !pte.valid() || (pte.permissions() as u8) ^ (XWRPermissions::ReadOnly as u8) == 0 {
            panic!(
                "Invalid PTE while trying to resolve virtual address {:x}",
                virt_addr
            );
        }
        // 4a. If pte.r = 1 || pte.x = 1, go to step 5
        if (pte.permissions() as u8) ^ (XWRPermissions::WriteOnly as u8) != 0 {
            // r/x
            // 5. Assume that the access is allowed.
            // 6. If i > 0 && pte.ppn[i - 1 : 0] != 0 raise fault
            // let pte_ppn_lookup = ppn_build_lookup(pte, levels, if i > 0 { i - 1 } else { 0 }, 0);
            // printk!("pte.ppn[i ( = {}) - 1 : 0] = {:x}", i, pte_ppn_lookup);
            // if i > 0 && pte_ppn_lookup != 0 {
            //     panic!(
            //         "Misaligned superpage while trying to resolve virtual address {:x}",
            //         virt_addr
            //     );
            // }
            // 7. Skipped since no operation is being done.
            // 8. Success, return the translated address.
            let pa_pgoff = virt_addr & !(PAGE_SIZE as u64 - 1);
            let mut ppn = alloc::vec![0u32; levels];
            // superpage translation
            if i > 0 {
                // copy vpn indices
                for j in (i - 1)..=0 {
                    ppn[j] = vpn[j] as u32;
                }
            }
            let pte_ppn = ppn_split_variable_sv39(pte);
            // copy ppn
            for j in (levels - 1)..=i {
                ppn[j] = pte_ppn[j];
            }
            // Reassemble ppn into address
            return reassemble_ppn(pa_pgoff as u16, ppn);
        }
        // 4b. i = i - 1, a = pte.ppn*PAGESIZE and go to step 2
        i -= 1;
        table = unsafe { &*(pte.phys_addr() as *const _) };
    }
    panic!("Shouldn't be here, page didn't resolve properly");
}

/// Helper used to implement RISC-V Privileged Specification, Section 4.3.2
fn reassemble_ppn(pgoff: u16, ppns: alloc::vec::Vec<u32>) -> u64 {
    let mut phys_addr = pgoff as u64;
    let mut factor = PAGE_SIZE as u64;
    for &ppn in ppns.iter() {
        phys_addr |= (ppn as u64) * factor;
        factor *= PAGE_SIZE as u64;
    }
    phys_addr
}

/// Helper used to implement RISC-V Privileged Specification, Section 4.3.2
fn vpn_split_variable(virt_addr: u64, levels: usize) -> alloc::vec::Vec<u16> {
    let mut virt_addr = virt_addr / PAGE_SIZE as u64;
    let mut vpns = alloc::vec![];
    for _ in 0..levels {
        vpns.push((virt_addr & (PAGE_SIZE as u64 - 1)) as u16);
        virt_addr /= PAGE_SIZE as u64;
    }
    vpns
}

/// Helper used to implement RISC-V Privileged Specification, Section 4.3.2
fn ppn_split_variable_sv39(pte: Sv39PTE) -> alloc::vec::Vec<u32> {
    alloc::vec![pte.ppn0() as u32, pte.ppn1() as u32, pte.ppn2()]
}

/// Helper used to implement RISC-V Privileged Specification, Section 4.3.2
fn ppn_build_lookup(pte: Sv39PTE, levels: usize, end: usize, start: usize) -> u64 {
    assert!(end <= levels, "Invalid bounds");
    let ppn = ppn_split_variable_sv39(pte);
    let mut phys_addr = 0u64;
    for index in start..=end {
        phys_addr |= ppn[index] as u64;
        phys_addr *= PAGE_SIZE as u64;
    }
    phys_addr
}

/// looks up a virtual address with the given root table
/// and returns the physical address
pub fn table_lookup2(table: &Sv39PageTable, virt_addr: u64) -> u64 {
    // let virt_addr = virt_addr & ((1 << 39) - 1);
    // let index = virt_addr / ONEGIG;
    let (vpn2, vpn1, vpn0) = split_virt_addr_sv39(virt_addr);
    let root_entry = table.entries[vpn2 as usize];
    if !root_entry.valid() {
        panic!(
            "Tried to lookup virtual address {:039x}, root entry {} is not valid!",
            virt_addr, vpn2
        );
    }
    if root_entry.permissions() != XWRPermissions::Pointer {
        // resolve here
        let phys_gaddr = root_entry.phys_addr();
        return phys_gaddr + ((vpn1 as u64) << 21) + ((vpn0 as u64) << 12) + (virt_addr % (1 << 9));
    }
    let level1_table = unsafe { &*(root_entry.phys_addr() as *const Sv39PageTable) };
    let level1_entry = level1_table.entries[vpn1 as usize];
    if !level1_entry.valid() {
        panic!(
            "Tried to lookup virtual address {:039x}, level 1 entry {} is not valid!",
            virt_addr, vpn1
        );
    }
    if level1_entry.permissions() != XWRPermissions::Pointer {
        let phys_gaddr = level1_entry.phys_addr();
        return phys_gaddr + ((vpn0 as u64) << 12) + (virt_addr % (1 << 9));
    }
    let level2_table = unsafe { &*(level1_entry.phys_addr() as *const Sv39PageTable) };
    let level2_entry = level2_table.entries[vpn0 as usize];
    if !level2_entry.valid() || level2_entry.permissions() == XWRPermissions::Pointer {
        panic!(
            "Tried to lookup virtual address {:039x}, level 2 entry {} is not valid!",
            virt_addr, vpn0
        );
    }
    let phys_gaddr = level2_entry.phys_addr();
    phys_gaddr + (virt_addr % (1 << 9))
}

/// enables paging with the given root table
///
/// # Safety
/// Only safe to call when the table's address is page-aligned,
/// and this code's physical page is identity mapped.
pub fn enable_paging(table: &Sv39PageTable) {
    // let ra2: u64;
    // unsafe {
    //     asm!("add {0}, ra, zero", out(reg) ra2);
    // }
    // printk!("Saved ra!");
    let addr = table as *const _ as usize;
    if addr % PAGE_SIZE != 0 {
        panic!("Table is not page-aligned");
    }
    let mode = 8u64; // sv39
    let ppn = (addr / PAGE_SIZE) as u64;
    printk!(
        "ppn = 0x{:x}, ppn << 12 = 0x{:x}, addr = 0x{:x}",
        ppn,
        ppn << 12,
        addr
    );
    // bottom 44 bits of ppn
    let satp_value = (mode << 60) | (ppn & ((1 << 44) - 1));
    unsafe {
        printk!("Setting satp to 0b{:064b}..", satp_value);
        // set satp
        asm!(
            "csrw satp, {0}",
            in(reg) satp_value,
        );
        printk!("set satp, going to sfence.vma now");
        asm!("sfence.vma");
        // printk!("going to sret now");
        // // // software + external
        // // let sie_value = (1 << 3) | (1 << 11);
        // // asm!(
        // //     "csrw sie, {0}",
        // //     in(reg) sie_value,
        // // );
        // // sret to paging_part2
        // asm!(
        //     "csrw sepc, {0}",
        //     in(reg) enable_paging_part2
        // );
        // asm!("sret");
    }
}

// static mut __PAGING_RA: u64 = 0;

// // empty, just return
// unsafe extern "C" fn enable_paging_part2() -> ! {
//     asm!("add ra, {0}, zero", in(reg) __PAGING_RA);
//     // // printk!("ra = {:x}", __PAGING_RA);
//     // let ra2: u64;
//     // asm!("add {0}, ra, zero", out(reg) ra2);
//     // printk!("ra = {:x}", ra2);
//     // printk!("addr of static = {:x}", &__PAGING_RA as *const _ as usize);
//     asm!("ret", options(noreturn));
// }

#[derive(BitfieldSpecifier, Clone, Copy, PartialEq, Eq)]
#[bits = 3]
enum XWRPermissions {
    Pointer = 0b000,
    ReadOnly = 0b001,
    WriteOnly = 0b010,
    ReadWrite = 0b011,
    ExecOnly = 0b100,
    ReadExec = 0b101,
    WriteExec = 0b110,
    ReadWriteExec = 0b111,
}

#[inline(never)]
pub const unsafe fn init() {}

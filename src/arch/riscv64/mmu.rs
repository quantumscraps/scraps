#![allow(dead_code)]

use modular_bitfield::prelude::*;

use crate::{physical_page_allocator::PAGE_SIZE, print, printk};

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

/// Splits a physical address into (PPN[2], PPN[1], PPN[0])
fn split_phys_addr(addr: u64) -> (u32, u16, u16) {
    let ppn0 = (addr >> 12) & ((1 << 10) - 1);
    let ppn1 = (addr >> 21) & ((1 << 10) - 1);
    let ppn2 = (addr >> 30) & ((1 << 27) - 1);
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

impl<PTE: Sized + Default> PageTable<PTE>
where
    [PTE; type_divide::<PTE>(PAGE_SIZE)]: Sized,
{
    pub fn init(&mut self) {
        for ent in self.entries.iter_mut() {
            *ent = Default::default();
        }
    }
}

pub type Sv39PageTable = PageTable<Sv39PTE>;

impl Sv39PageTable {
    pub fn print(&self) {
        for (i, ent) in self.entries.iter().enumerate() {
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
            print!(" -> 0x{:x}\n", ent.phys_addr() << 12);
        }
    }
}

const ONEGIG: u64 = 0x40000000;

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

/// looks up a virtual address with the given root table
/// and returns the physical address
///
/// only the bottom 39 bits of the virtual address are used
/// and the physical address is masked to 56 bits
///
/// currently, only 1g gigagpages are supported
pub fn table_lookup(table: &Sv39PageTable, virt_addr: u64) -> u64 {
    let virt_addr = virt_addr & ((1 << 40) - 1);
    let index = virt_addr / ONEGIG;
    let entry = table.entries[index as usize];
    if !entry.valid() {
        panic!(
            "Tried to lookup virtual address {:039x}, entry {} is not valid!",
            virt_addr, index
        );
    } else {
        // translate the offset from virt address to physical address
        let phys_gaddr = entry.phys_addr();
        let res = phys_gaddr + (virt_addr % ONEGIG);
        res & ((1 << 57) - 1)
    }
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

#[derive(BitfieldSpecifier, Clone, Copy)]
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

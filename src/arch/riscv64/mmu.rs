#![allow(dead_code)]

use modular_bitfield::prelude::*;

use crate::{
    mmu::{PageTable, PagingSetup, Permissions},
    physical_page_allocator::ALLOCATOR,
    print, printk,
};

// 1 << 12
// 0x100
pub const PAGE_SIZE: usize = 4096;

pub const ONEGIG: u64 = 0x40000000;

#[repr(u8)]
pub enum PagingMode {
    Sv39 = 8,
}

pub trait Sv {
    type PTE: SvPTE;
    type Table: SvTable<PTE = Self::PTE, Sv = Self>;
    const MODE: PagingMode;
}

pub trait SvPTE: Sized {
    fn from_physical_addr(addr: u64) -> Self;
    fn physical_addr(&self) -> u64;
    fn valid(&self) -> bool;
    fn global(&self) -> bool;
    fn permissions(&self) -> XWRPermissions;
}

pub trait SvTable: PageTable {
    type Sv: Sv;
    type PTE: SvPTE;
    const ENTRIES: usize;

    /// Unsafely casts a *mut u8 to this paging system's page table.
    unsafe fn cast_page_table<'a>(ptr: *mut u8) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    /// Maps a 1GiB gigapage by rounding the given address.
    fn map_gigapage(&mut self, virt_addr: u64, phys_addr: u64, permissions: XWRPermissions);

    /// Maps a 4KiB page by rounding the given address.
    fn map_page(&mut self, virt_addr: u64, phys_addr: u64, permissions: XWRPermissions);

    /// Maps a range of pages using [map_page]
    /// The begin address is rounded down, and the end address is rounded up.
    fn map_page_range(
        &mut self,
        virt_addr_begin: u64,
        virt_addr_end: u64,
        phys_addr_begin: u64,
        permissions: XWRPermissions,
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
            self.map_page(virt_addr, phys_addr, permissions);
        }
    }

    /// Maps a paging setup.
    fn map_page_setup(&mut self, setup: &PagingSetup) {
        for (&virt_base, &(start, end, permissions)) in setup.mappings.iter() {
            let permissions_converted = {
                let mut p = 0;
                if permissions.contains(Permissions::Read) {
                    p |= XWRPermissions::ReadOnly as u8;
                }
                if permissions.contains(Permissions::Write) {
                    p |= XWRPermissions::WriteOnly as u8;
                }
                if permissions.contains(Permissions::Execute) {
                    p |= XWRPermissions::ExecOnly as u8;
                }
                if p == 0 {
                    // Unreachable since you cannot construct a Permissions
                    // instance with a value of zero
                    unreachable!()
                } else {
                    // safe: The only bits that can be set are valid
                    unsafe { core::mem::transmute::<_, XWRPermissions>(p) }
                }
            };
            self.map_page_range(
                virt_base as u64,
                (end - start + virt_base) as u64,
                start as u64,
                permissions_converted,
            );
        }
    }

    /// Looks up a virtual address.
    fn virt_to_phys(&self, virt_addr: u64) -> u64;

    fn entries(&self) -> &[Self::PTE];

    fn print(&self) {
        self.print_indented(0);
    }

    fn print_indented(&self, indent: usize) {
        for (i, ent) in self.entries().iter().enumerate() {
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
            print!(" -> 0x{:x}\n", ent.physical_addr());
            // If this PTE is a indirection, print out the next level
            if ent.valid() && perms == XWRPermissions::Pointer {
                let next_level_table = unsafe { &*(ent.physical_addr() as *const Self) };
                next_level_table.print_indented(indent + 1);
            }
        }
    }
}

pub struct Sv39;

impl Sv for Sv39 {
    type PTE = Sv39PTE;
    type Table = Sv39Table;
    const MODE: PagingMode = PagingMode::Sv39;
}

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

impl SvPTE for Sv39PTE {
    fn from_physical_addr(addr: u64) -> Self {
        let (ppn2, ppn1, ppn0) = split_phys_addr_sv39(addr);
        Self::new().with_ppn2(ppn2).with_ppn1(ppn1).with_ppn0(ppn0)
    }

    fn physical_addr(&self) -> u64 {
        unsplit_phys_addr_sv39((self.ppn2(), self.ppn1(), self.ppn0()))
    }

    fn valid(&self) -> bool {
        self.valid()
    }

    fn global(&self) -> bool {
        self.global()
    }

    fn permissions(&self) -> XWRPermissions {
        self.permissions()
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
fn split_phys_addr_sv39(addr: u64) -> (u32, u16, u16) {
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
fn unsplit_phys_addr_sv39(parts: (u32, u16, u16)) -> u64 {
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
const fn type_divide<T>(n: usize) -> usize {
    let t_sz = core::mem::size_of::<T>();
    if n % t_sz != 0 {
        panic!("Size of type does not divide N evenly");
    }
    if t_sz == 0 {
        panic!("Trying to divide a ZST");
    }
    n / t_sz
}

pub struct Sv39Table {
    entries: [<Self as SvTable>::PTE; Self::ENTRIES],
}

impl<T: SvTable<Sv = U>, U: Sv<Table = T>> PageTable for T {
    fn print(&self) {
        SvTable::print(self);
    }

    unsafe fn enable(&self) {
        enable_paging::<U>(self);
    }

    fn virt_to_phys(&self, virt_addr: usize) -> usize {
        SvTable::virt_to_phys(self, virt_addr as _) as _
    }
}

impl SvTable for Sv39Table {
    type Sv = Sv39;
    type PTE = Sv39PTE;
    const ENTRIES: usize = type_divide::<Self::PTE>(PAGE_SIZE);

    fn map_gigapage(&mut self, virt_addr: u64, phys_addr: u64, permissions: XWRPermissions) {
        // let virt_addr = virt_addr & !(onegig - 1);
        // round the address down
        let index = virt_addr / ONEGIG;
        printk!("Mapping root index {}", index);
        // mask out 1g of phys_addr
        let phys_addr2 = phys_addr & !(ONEGIG - 1);
        // set flags to vgrwx for now
        self.entries[index as usize] = Sv39PTE::from_physical_addr(phys_addr2)
            .with_valid(true)
            .with_global(true)
            .with_permissions(permissions);
    }

    /// maps a 4k page by rounding the given addr
    fn map_page(&mut self, virt_addr: u64, phys_addr: u64, permissions: XWRPermissions) {
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
        let root_entry = &mut self.entries[vpn2 as usize];
        // if invalid, allocate page
        // otherwise, use new pointer
        let level1_table = if root_entry.valid() {
            unsafe { &mut *(root_entry.physical_addr() as *mut Self) }
        } else {
            // allocate page
            let new_addr = unsafe { &mut ALLOCATOR }
                .try_zallocate(PAGE_SIZE)
                .expect("Failed to allocate page!") as u64;
            // set entry
            *root_entry = Sv39PTE::from_physical_addr(new_addr)
                .with_valid(true)
                .with_permissions(XWRPermissions::Pointer);
            let table = unsafe { &mut *(new_addr as *mut Self) };
            table
        };
        // same thing for level2
        let level1_entry = &mut level1_table.entries[vpn1 as usize];
        let level2_table = if level1_entry.valid() {
            unsafe { &mut *(level1_entry.physical_addr() as *mut Self) }
        } else {
            // allocate page
            let new_addr = unsafe { &mut ALLOCATOR }
                .try_zallocate(PAGE_SIZE)
                .expect("Failed to allocate page!") as u64;
            // set entry
            *level1_entry = Sv39PTE::from_physical_addr(new_addr)
                .with_valid(true)
                .with_permissions(XWRPermissions::Pointer);
            let table = unsafe { &mut *(new_addr as *mut Self) };
            table
        };
        let level2_entry = &mut level2_table.entries[vpn0 as usize];
        // check that the entry is not valid already
        if level2_entry.valid() {
            panic!("Trying to overwrite a valid PTE entry");
        }
        // set leaf
        *level2_entry = Sv39PTE::from_physical_addr(phys_addr2)
            .with_valid(true)
            .with_global(true)
            .with_permissions(permissions);
    }

    fn virt_to_phys(&self, virt_addr: u64) -> u64 {
        // let virt_addr = virt_addr & ((1 << 39) - 1);
        // let index = virt_addr / ONEGIG;
        let (vpn2, vpn1, vpn0) = split_virt_addr_sv39(virt_addr);
        let root_entry = self.entries[vpn2 as usize];
        if !root_entry.valid() {
            panic!(
                "Tried to lookup virtual address {:039x}, root entry {} is not valid!",
                virt_addr, vpn2
            );
        }
        if root_entry.permissions() != XWRPermissions::Pointer {
            // resolve here
            let phys_gaddr = root_entry.physical_addr();
            return phys_gaddr
                + ((vpn1 as u64) << 21)
                + ((vpn0 as u64) << 12)
                + (virt_addr % (1 << 9));
        }
        let level1_table = unsafe { &*(root_entry.physical_addr() as *const Self) };
        let level1_entry = level1_table.entries[vpn1 as usize];
        if !level1_entry.valid() {
            panic!(
                "Tried to lookup virtual address {:039x}, level 1 entry {} is not valid!",
                virt_addr, vpn1
            );
        }
        if level1_entry.permissions() != XWRPermissions::Pointer {
            let phys_gaddr = level1_entry.physical_addr();
            return phys_gaddr + ((vpn0 as u64) << 12) + (virt_addr % (1 << 9));
        }
        let level2_table = unsafe { &*(level1_entry.physical_addr() as *const Self) };
        let level2_entry = level2_table.entries[vpn0 as usize];
        if !level2_entry.valid() || level2_entry.permissions() == XWRPermissions::Pointer {
            panic!(
                "Tried to lookup virtual address {:039x}, level 2 entry {} is not valid!",
                virt_addr, vpn0
            );
        }
        let phys_gaddr = level2_entry.physical_addr();
        phys_gaddr + (virt_addr % (1 << 9))
    }

    fn entries(&self) -> &[Self::PTE] {
        &self.entries
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
    printk!("setting mepc to part2...");
    asm!(
        "csrw mepc, {0}",
        in(reg) return_to,
    );
    printk!("mret...");
    asm!("mret", options(noreturn));
}

/// Enables paging with the given root table
///
/// # Safety
/// Only safe to call when the table's address is page-aligned,
/// and this code's physical page is identity mapped.
pub unsafe fn enable_paging<PageSystem: Sv + ?Sized>(table: &PageSystem::Table) {
    let addr = table as *const _ as usize;
    if addr % PAGE_SIZE != 0 {
        panic!("Table is not page-aligned");
    }
    let mode = PageSystem::MODE as u64;
    let ppn = (addr / PAGE_SIZE) as u64;
    printk!(
        "ppn = 0x{:x}, ppn << 12 = 0x{:x}, addr = 0x{:x}",
        ppn,
        ppn << 12,
        addr
    );
    // bottom 44 bits of ppn
    let satp_value = (mode << 60) | (ppn & ((1 << 44) - 1));
    // unsafe {
    printk!("Setting satp to 0b{:064b}..", satp_value);
    // set satp
    asm!(
        "csrw satp, {0}",
        in(reg) satp_value,
    );
    printk!("set satp, going to sfence.vma now");
    asm!("sfence.vma");
    // }
}

#[derive(BitfieldSpecifier, Clone, Copy, PartialEq, Eq)]
#[bits = 3]
pub enum XWRPermissions {
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

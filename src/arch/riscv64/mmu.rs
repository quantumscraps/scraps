#![allow(dead_code)]

use modular_bitfield::prelude::*;

use crate::{
    link_var,
    mmu::{PageTable, Permissions, HIGHER_HALF_BASE},
    physical_page_allocator::ALLOCATOR,
    print, printk, STDOUT,
};

// 1 << 12
// 0x100
pub const PAGE_SIZE: usize = 4096;

pub const ONEGIG: usize = 0x40000000;

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
    fn from_physical_addr(addr: usize) -> Self;
    fn physical_addr(&self) -> usize;
    fn valid(&self) -> bool;
    fn global(&self) -> bool;
    fn permissions(&self) -> XWRPermissions;
}

pub trait SvTable: PageTable {
    type Sv: Sv;
    type PTE: SvPTE;
    const ENTRIES: usize;

    /// Unsafely casts a *mut u8 to this paging system's page table.
    ///
    /// # Safety
    /// Only safe if the given pointer points to an unused page.
    unsafe fn cast_page_table<'a>(ptr: *mut u8) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    /// Maps a 1GiB gigapage by rounding the given address.
    fn map_gigapage(&mut self, virt_addr: usize, phys_addr: usize, permissions: XWRPermissions);

    /// Maps a 4KiB page by rounding the given address.
    fn map_page(&mut self, virt_addr: usize, phys_addr: usize, permissions: XWRPermissions);

    /// Unmaps a 4KiB page by rounding the given address.
    fn unmap_page(&mut self, virt_addr: usize);

    /// Maps a 1GiB gigapage by rounding the given address.
    fn unmap_gigapage(&mut self, virt_addr: usize);

    /// Makes a deep clone of this page table.
    fn deep_clone(&self, current_pt: &Self, old_base: usize, new_base: usize) -> &mut Self;

    /// Deep frees this page table.
    fn deep_free(self: &mut Self);

    /// Maps a range of pages using [map_page]
    /// The begin address is rounded down, and the end address is rounded up.
    fn map_page_range(
        &mut self,
        virt_addr_begin: usize,
        virt_addr_end: usize,
        phys_addr_begin: usize,
        permissions: XWRPermissions,
    ) {
        let virt_offset = virt_addr_begin / PAGE_SIZE;
        let phys_offset = phys_addr_begin / PAGE_SIZE;
        let virt_addr_begin = virt_addr_begin & !(PAGE_SIZE - 1);
        let virt_addr_end = if virt_addr_end % PAGE_SIZE > 0 {
            ((virt_addr_end) & !(PAGE_SIZE - 1)) + PAGE_SIZE
        } else {
            virt_addr_end
        };
        let len = (virt_addr_end - virt_addr_begin) / PAGE_SIZE;
        printk!("Mapping {} pages", len);
        for i in 0..len {
            let virt_addr = (i + virt_offset) * PAGE_SIZE;
            let phys_addr = (i + phys_offset) * PAGE_SIZE;
            // printk!("Mapping {:x} -> {:x}", virt_addr, phys_addr);
            self.map_page(virt_addr, phys_addr, permissions);
        }
    }

    /// Unmaps a range of pages using [map_page]
    /// Same behavior as [map_page_range].
    fn unmap_page_range(&mut self, virt_addr_begin: usize, virt_addr_end: usize) {
        let virt_offset = virt_addr_begin / PAGE_SIZE;
        // let phys_offset = phys_addr_begin / PAGE_SIZE;
        let virt_addr_begin = virt_addr_begin & !(PAGE_SIZE - 1);
        let virt_addr_end = if virt_addr_end % PAGE_SIZE > 0 {
            ((virt_addr_end) & !(PAGE_SIZE - 1)) + PAGE_SIZE
        } else {
            virt_addr_end
        };
        let len = (virt_addr_end - virt_addr_begin) / PAGE_SIZE;
        printk!("Unmapping {} pages", len);
        for i in 0..len {
            let virt_addr = (i + virt_offset) * PAGE_SIZE;
            // let phys_addr = (i + phys_offset) * page_size_u64;
            // printk!("Unmapping {:x}", virt_addr);
            self.unmap_page(virt_addr);
        }
    }

    /// Looks up a virtual address.
    fn virt_to_phys(&self, virt_addr: usize) -> usize;

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
                if perms.contains(XWRPermissions::Read) {
                    print!("R");
                } else {
                    print!(" ");
                }
                if perms.contains(XWRPermissions::Write) {
                    print!("W");
                } else {
                    print!(" ");
                }
                if perms.contains(XWRPermissions::Execute) {
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
    fn from_physical_addr(addr: usize) -> Self {
        let (ppn2, ppn1, ppn0) = split_phys_addr_sv39(addr);
        Self::new().with_ppn2(ppn2).with_ppn1(ppn1).with_ppn0(ppn0)
    }

    fn physical_addr(&self) -> usize {
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
fn split_virt_addr_sv39(addr: usize) -> (u16, u16, u16) {
    let vpn0 = (addr >> 12) & ((1 << 9) - 1);
    let vpn1 = (addr >> 21) & ((1 << 9) - 1);
    let vpn2 = (addr >> 30) & ((1 << 9) - 1);
    (vpn2 as u16, vpn1 as u16, vpn0 as u16)
}

/// Splits a physical address into (PPN[2], PPN[1], PPN[0])
fn split_phys_addr_sv39(addr: usize) -> (u32, u16, u16) {
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
fn unsplit_phys_addr_sv39(parts: (u32, u16, u16)) -> usize {
    let ppn2 = parts.0 as usize;
    let ppn1 = parts.1 as usize;
    let ppn0 = parts.2 as usize;
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

impl Sv39Table {
    const fn new() -> Self {
        Self {
            entries: [Sv39PTE::new(); Self::ENTRIES],
        }
    }
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

    fn map_gigapage(&mut self, virt_addr: usize, phys_addr: usize, permissions: XWRPermissions) {
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

    fn unmap_gigapage(&mut self, virt_addr: usize) {
        // let virt_addr = virt_addr & !(onegig - 1);
        // round the address down
        let index = virt_addr / ONEGIG;
        printk!("Unmapping root index {}", index);
        // mask out 1g of phys_addr
        // let phys_addr2 = phys_addr & !(ONEGIG - 1);
        // set flags to vgrwx for now
        self.entries[index as usize].set_valid(false);
    }

    /// maps a 4k page by rounding the given addr
    fn map_page(&mut self, virt_addr: usize, phys_addr: usize, permissions: XWRPermissions) {
        // mask out page_size of phys_addr
        let phys_addr2 = phys_addr & !(PAGE_SIZE - 1);
        // split virt addr
        let (vpn2, vpn1, vpn0) = split_virt_addr_sv39(virt_addr);
        // printk!(
        //     "root index = {}, level1 index = {}, level2 index = {}",
        //     vpn2,
        //     vpn1,
        //     vpn0
        // );
        let root_entry = &mut self.entries[vpn2 as usize];
        // if invalid, allocate page
        // otherwise, use new pointer
        let level1_table = if root_entry.valid() {
            unsafe { &mut *(root_entry.physical_addr() as *mut Self) }
        } else {
            // allocate page
            let new_addr = unsafe { &mut ALLOCATOR }
                .try_zallocate(PAGE_SIZE)
                .expect("Failed to allocate page!") as usize;
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
                .expect("Failed to allocate page!") as usize;
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

    /// maps a 4k page by rounding the given addr
    fn unmap_page(&mut self, virt_addr: usize) {
        // mask out page_size of phys_addr
        // let phys_addr2 = phys_addr & !(PAGE_SIZE as u64 - 1);
        // split virt addr
        let (vpn2, vpn1, vpn0) = split_virt_addr_sv39(virt_addr);
        // printk!(
        //     "root index = {}, level1 index = {}, level2 index = {}",
        //     vpn2,
        //     vpn1,
        //     vpn0
        // );
        let root_entry = &mut self.entries[vpn2 as usize];
        // if invalid, allocate page
        // otherwise, use new pointer
        let level1_table = if root_entry.valid() {
            unsafe { &mut *(root_entry.physical_addr() as *mut Self) }
        } else {
            return;
        };
        // same thing for level2
        let level1_entry = &mut level1_table.entries[vpn1 as usize];
        let level2_table = if level1_entry.valid() {
            unsafe { &mut *(level1_entry.physical_addr() as *mut Self) }
        } else {
            return;
        };
        let level2_entry = &mut level2_table.entries[vpn0 as usize];
        // set leaf
        // just set invalid
        level2_entry.set_valid(false);
    }

    fn virt_to_phys(&self, virt_addr: usize) -> usize {
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
                + ((vpn1 as usize) << 21)
                + ((vpn0 as usize) << 12)
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
            return phys_gaddr + ((vpn0 as usize) << 12) + (virt_addr % (1 << 9));
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

    fn deep_clone(&self, current_pt: &Self, old_base: usize, new_base: usize) -> &mut Self {
        let root_clone = unsafe {
            Self::cast_page_table(
                ALLOCATOR
                    .try_zallocate(PAGE_SIZE)
                    .expect("Failed to allocate page")
                    .offset((new_base - old_base) as _),
            )
        };
        root_clone.entries = self.entries;
        for level1_entry in root_clone.entries.iter_mut() {
            if level1_entry.permissions() == XWRPermissions::Pointer {
                // Clone this level
                let level1_clone = unsafe {
                    Self::cast_page_table(
                        ALLOCATOR
                            .try_zallocate(PAGE_SIZE)
                            .expect("Failed to allocate page")
                            .offset((new_base - old_base) as _),
                    )
                };
                let old_level1 =
                    unsafe { Self::cast_page_table(level1_entry.physical_addr() as _) };
                level1_clone.entries = old_level1.entries;

                for level2_entry in level1_clone.entries.iter_mut() {
                    if level2_entry.permissions() == XWRPermissions::Pointer {
                        // clone this level
                        let level2_clone = unsafe {
                            Self::cast_page_table(
                                ALLOCATOR
                                    .try_zallocate(PAGE_SIZE)
                                    .expect("Failed to allocate page")
                                    .offset((new_base - old_base) as _),
                            )
                        };
                        let old_level2 =
                            unsafe { Self::cast_page_table(level2_entry.physical_addr() as _) };
                        level2_clone.entries = old_level2.entries;

                        // Level 3 is always leaf, no need to iterate

                        // Set new addr
                        // Set new addr
                        *level2_entry = Sv39PTE::from_physical_addr(SvTable::virt_to_phys(
                            current_pt,
                            level2_clone as *mut _ as usize,
                        ))
                        .with_valid(level2_entry.valid())
                        .with_global(level2_entry.global())
                        .with_permissions(level2_entry.permissions());
                    }
                }

                // Set new addr
                *level1_entry = Sv39PTE::from_physical_addr(SvTable::virt_to_phys(
                    current_pt,
                    level1_clone as *mut _ as usize,
                ))
                .with_valid(level1_entry.valid())
                .with_global(level1_entry.global())
                .with_permissions(level1_entry.permissions());
            }
        }

        root_clone
    }

    fn deep_free(self: &mut Self) {
        for level1_entry in &self.entries {
            if level1_entry.permissions() == XWRPermissions::Pointer {
                // Free inner entries first
                let level1_table =
                    unsafe { Self::cast_page_table(level1_entry.physical_addr() as _) };

                for level2_entry in &level1_table.entries {
                    if level2_entry.permissions() == XWRPermissions::Pointer {
                        // Free this table, level3 is always leaf
                        unsafe { &mut ALLOCATOR }.deallocate(
                            level2_entry.physical_addr() as _,
                            core::mem::size_of::<Self>(),
                        );
                    }
                }

                // Free level1 table
                unsafe { &mut ALLOCATOR }.deallocate(
                    level1_entry.physical_addr() as _,
                    core::mem::size_of::<Self>(),
                );
            }
        }

        // Free ourself
        unsafe { &mut ALLOCATOR }.deallocate(self as *mut _ as _, core::mem::size_of::<Self>());
    }
}

/// Enables S-mode
///
/// # Safety
/// The return address and trap vector must be pointing to a valid instruction.
/// Additionally, the trap vector's address must be aligned to 4 bytes.
pub unsafe fn enable_smode(return_to: usize, trap_vector: usize) {
    assert!(trap_vector % 4 == 0, "Trap vector is not properly aligned!");
    printk!("setting mstatus to s-mode, zeroing mie");
    let mstatus_val = 1 << 11 | 1 << 7;
    asm!("csrw mstatus, {0}", in(reg) mstatus_val);
    // bottom two bits are mode, and leaving at zero
    // is direct, which works for now
    let _mtvec_val = trap_vector << 2;
    //printk!("setting mtvec to given value...");
    //asm!("csrw mtvec, {0}", in(reg) mtvec_val);
    //asm!("csrw mie, zero");
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
    let mode = PageSystem::MODE as usize;
    let ppn = addr / PAGE_SIZE;
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

pub unsafe fn enable_paging2<PageSystem: Sv + ?Sized>(phys_addr: usize) {
    let addr = phys_addr;
    if addr % PAGE_SIZE != 0 {
        panic!("Table is not page-aligned");
    }
    let mode = PageSystem::MODE as usize;
    let ppn = addr / PAGE_SIZE;
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

#[allow(non_upper_case_globals)]
mod permissions_inner {
    use super::Permissions;
    use modular_bitfield::{error::*, Specifier};

    bitflags::bitflags! {
        pub struct XWRPermissions: u8 {
            const Pointer = 0b000;
            const Read = 0b001;
            const Write = 0b010;
            const Execute = 0b100;
        }
    }

    impl Specifier for XWRPermissions {
        type Bytes = u8;
        type InOut = Self;
        const BITS: usize = 3;

        fn into_bytes(input: Self::InOut) -> Result<Self::Bytes, OutOfBounds> {
            Ok(input.bits)
        }

        fn from_bytes(bytes: Self::Bytes) -> Result<Self::InOut, InvalidBitPattern<Self::Bytes>> {
            Self::from_bits(bytes).ok_or(InvalidBitPattern::new(bytes))
        }
    }

    impl From<Permissions> for XWRPermissions {
        fn from(permissions: Permissions) -> Self {
            let mut p = XWRPermissions::empty();
            if permissions.contains(Permissions::Read) {
                p |= XWRPermissions::Read;
            }
            if permissions.contains(Permissions::Write) {
                p |= XWRPermissions::Write;
            }
            if permissions.contains(Permissions::Execute) {
                p |= XWRPermissions::Execute;
            }
            if p.is_empty() {
                // Unreachable since an empty Permissions cannot be
                // inserted into a PagingSetup
                unreachable!()
            } else {
                p
            }
        }
    }
}
pub use permissions_inner::XWRPermissions;

use super::trap::TrapFrame;

/// Root page table, for initial higher half boot.
/// Put in its own section so it is guaranteed to be in the
/// first 1GiB of the kernel.
#[link_section = ".data.rpt"] // rpt = root page table
#[no_mangle]
#[allow(non_upper_case_globals)]
pub static mut __root_page_table: Sv39Table = Sv39Table::new();

#[inline(never)]
pub unsafe extern "C" fn init(return_to: usize, ra: usize, a0: usize, a1: usize) -> ! {
    // let ra: usize;
    // asm!("mv {0}, ra", out(reg) ra);
    // Create a page table with kernel mapped to higher half, and STDOUT identity mapped
    // let page_table_ptr = ALLOCATOR
    //     .try_zallocate(PAGE_SIZE)
    //     .expect("Failed to allocate page");
    // let page_table = Sv39Table::cast_page_table(page_table_ptr);
    // map in kernel
    link_var!(__kern_start, __kern_end);
    let kern_start = &__kern_start as *const _ as usize;
    // identity map until we disable it later
    __root_page_table.map_gigapage(kern_start, kern_start, Permissions::RWX.into());
    __root_page_table.map_gigapage(HIGHER_HALF_BASE as _, kern_start, Permissions::RWX.into());
    // page_table.map_page_range(kern_start, kern_end, kern_start, Permissions::RWX.into());
    // page_table.map_page_range(
    //     HIGHER_HALF_BASE as _,
    //     kern_end - kern_start + (HIGHER_HALF_BASE as u64),
    //     kern_start,
    //     Permissions::RWX.into(),
    // );
    // map in stdout
    use crate::driver_interfaces::Console;
    if let Some(ref stdout) = *STDOUT.get_mut() {
        let base = stdout.base_address();
        // map gigapage since no alloc setup yet
        __root_page_table.map_gigapage(base as _, base as _, Permissions::RW.into());
    }
    __root_page_table.enable();
    // test if paging worked
    let ptest = *(((&crate::PAGING_TEST as *const _ as usize) - kern_start + HIGHER_HALF_BASE)
        as *const usize);
    printk!("PAGING_TEST from higher half: {:x}", ptest);
    // enable page table and jump to higher half
    let jump_to = (higher_half_mmu_cont as usize) - kern_start + HIGHER_HALF_BASE;
    printk!("Jumping to {:x}", jump_to);
    // setup stack and gp too
    let gp: usize;
    let sp: usize;
    // let ra: u64;
    asm!("mv {0}, gp", out(reg) gp);
    asm!("mv {0}, sp", out(reg) sp);
    // asm!("mv {0}, ra", out(reg) ra);
    asm!("mv gp, {0}", in(reg) gp - kern_start + HIGHER_HALF_BASE);
    asm!("mv sp, {0}", in(reg) sp - kern_start + HIGHER_HALF_BASE);
    // asm!("mv t5, {0}", in(reg) ra - kern_start + HIGHER_HALF_BASE, lateout("t5") _);
    // printk!("New ra = {:x}", ra - kern_start + (HIGHER_HALF_BASE as u64));
    let f = core::mem::transmute::<_, unsafe extern "C" fn(usize, usize)>(jump_to);
    asm!("jalr {0}", in(reg) f, in("a0") kern_start, in("a1") return_to, in("a2") ra, in("a3") a0, in("a4") a1, options(noreturn));
    // f(kern_start, kern_end);
}

unsafe extern "C" fn higher_half_mmu_cont(
    old_kern_start: usize,
    jump_to: usize,
    ra: usize,
    a0: usize,
    a1: usize,
) -> ! {
    // asm!("sub ra, ra, {0}", "add ra, ra, {1}", in(reg) old_kern_start, in(reg) HIGHER_HALF_BASE, lateout("ra") _);
    // Reserve t5
    // asm!("", lateout("t5") _);
    printk!("Made it to higher half");
    // let satp: usize;
    // asm!("csrr {0}, satp", out(reg) satp);
    // Fix stvec before enabling new table
    let stvec: usize;
    asm!("csrr {0}, stvec", out(reg) stvec);
    asm!("csrw stvec, {0}", in(reg) stvec - old_kern_start + HIGHER_HALF_BASE);
    // Fix sscratch
    let sscratch: usize;
    asm!("csrr {0}, sscratch", out(reg) sscratch);
    let new_sscratch = sscratch - old_kern_start + HIGHER_HALF_BASE;
    printk!("New sscratch = {:x}", new_sscratch);
    asm!("csrw sscratch, {0}", in(reg) new_sscratch);
    // Fix trap stack!
    let trap_frame = &mut *(new_sscratch as *mut TrapFrame);
    trap_frame.trap_stack = trap_frame
        .trap_stack
        .offset((HIGHER_HALF_BASE - old_kern_start) as _);
    // (0 as *const usize).read_volatile();
    // let page_table_ptr = ((satp & ((1 << 44) - 1)) * PAGE_SIZE) - old_kern_start + HIGHER_HALF_BASE;
    // let page_table = Sv39Table::cast_page_table(page_table_ptr as _);
    // Create a copy!
    // printk!("Value of stack satp = {:x}", &satp as *const _ as usize);
    // printk!("Value of page_table_ptr = {:x}", page_table_ptr);
    // let new_page_table =
    //     page_table.deep_clone(&page_table, old_kern_start, HIGHER_HALF_BASE as u64);
    // Unmap identity kernel
    // link_var!(__kern_start, __kern_end);
    // let kern_start = &__kern_start as *const _ as u64;
    // let kern_end = &__kern_end as *const _ as u64;
    printk!("Gonna unmap old kern gigapage");
    __root_page_table.unmap_gigapage(old_kern_start);
    // switch to new page table
    // let new_phys_addr = PageTable::virt_to_phys(page_table, &new_page_table as *const _ as usize);
    // printk!("New phys addr = {:x}", new_phys_addr);
    // enable_paging2::<Sv39>(new_phys_addr);
    printk!("It worked!");
    // Free old table
    // page_table.deep_free();
    // // done! free old page table
    // ALLOCATOR.deallocate(
    //     page_table as *mut _ as *mut u8,
    //     core::mem::size_of::<Sv39Table>(),
    // );
    // return to old address
    // asm!("jalr t5");
    // return to given addr
    asm!("jalr {0}", in(reg) jump_to, in("ra") ra, in("a0") a0, in("a1") a1, options(noreturn));
}

use alloc::collections::BTreeMap;

/// MMU abstraction that can be evaluated by any paging system.
pub struct PagingSetup {
    //                     virt_base
    //                     |       phys_start
    //                     |       |      phys_end
    //                     |       |      |      permissions
    //                     |       |      |      |
    mappings: BTreeMap<usize, (usize, usize, Permissions)>,
}

impl PagingSetup {
    pub fn new() -> Self {
        Self {
            mappings: BTreeMap::new(),
        }
    }

    pub fn map(
        &mut self,
        virt_base: usize,
        phys_start: usize,
        phys_end: usize,
        permissions: Permissions,
    ) {
        assert!(!permissions.is_empty(), "Empty permissions are invalid");
        assert_eq!(
            self.mappings
                .insert(virt_base, (phys_start, phys_end, permissions)),
            None,
            "Trying to map two ranges at same virt_base!"
        );
    }

    pub fn mappings(&self) -> &BTreeMap<usize, (usize, usize, Permissions)> {
        &self.mappings
    }
}

/// Common interface implemented by all page tables.
pub trait PageTable: Sized {
    /// Creates an instance of this page table from a page setup.
    fn from_page_setup<'a, 'b>(setup: &'a PagingSetup) -> &'b mut Self;

    /// Prints a fancy representation of this page table.
    fn print(&self);

    /// Enables paging with this page table.
    ///
    /// # Safety
    /// Only safe to call if the executing page is identity mapped.
    unsafe fn enable(&self);

    /// Converts a virtual address to a physical address.
    fn virt_to_phys(&self, virt_addr: usize) -> usize;
}

// Hack to make the allow work
#[allow(non_upper_case_globals)]
mod permissions_inner {
    bitflags::bitflags! {
        /// Abstract representation of page permissions.
        pub struct Permissions: u8 {
            const Read = 1 << 0;
            const Write = 1 << 1;
            const Execute = 1 << 2;
            const RW = Self::Read.bits | Self::Write.bits;
            const WX = Self::Write.bits | Self::Execute.bits;
            // RX doesn't make sense
            const RWX = Self::Read.bits | Self::Write.bits | Self::Execute.bits;
        }
    }
}

pub use permissions_inner::Permissions;

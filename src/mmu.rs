/// Common interface implemented by all page tables.
pub trait PageTable: Sized {
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

pub const HIGHER_HALF_BASE: usize = 0xC0000000;

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

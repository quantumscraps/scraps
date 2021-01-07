use crate::{
    arch::mmu::{Sv, Sv39},
    mmu::{PageTable, PagingSetup},
};

// Dumped dtb with `-M virt,dumpdtb=virt.out` to check timebase_freq
// which is 10,000,000
// Linux also uses HZ which is default to 1000
// 1,000,000,000 / (timebase_freq * HZ) < 0
// ~= 0.25
// therefore use TICKS_PER_NANO = 2.5
// (this seems pretty correct based on testing)
pub const TICKS_PER_NANO: u64 = 3; // 10 / 4 ~= 3, should probably add floats though
pub const NANOS_PER_TICK: u64 = 1;
pub const HAS_RDTIME: bool = false;

pub fn map_page_setup(setup: &PagingSetup) -> &mut impl PageTable {
    <Sv39 as Sv>::Table::from_page_setup(setup)
}

pub const HEAP_SIZE: usize = 0x100000; // PAGE_SIZE * 1048576; // 1m allocations=

pub use crate::arch::mmu::PAGE_SIZE;

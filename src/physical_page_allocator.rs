pub use crate::bsp::{HEAP_SIZE, PAGE_SIZE};
use crate::{link_var, print, printk, println};
use core::alloc::GlobalAlloc;

link_var!(__heap_start);

// const_evalutable_checked cannot evaluate this expression inline yet.
const fn pages_subdivide(size: usize) -> usize {
    // requires const_panic
    assert!(
        size % PAGE_SIZE == 0,
        "heap size must be a multiple of page size"
    );
    size / PAGE_SIZE
}

/// Page flags
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum PageFlags {
    Free = 1 << 0,
    Taken = 1 << 1,
}

impl PageFlags {
    pub const fn val(&self) -> u8 {
        *self as u8
    }
}

/// Physical page allocator weeee
pub struct PhysicalPageAllocator<const PAGES: usize>
where
    [u8; pages_subdivide(PAGES)]: Sized,
{
    start: usize,
    descriptors: [u8; pages_subdivide(PAGES)],
}

// TODO: write an actual allocator that's efficient
// this literally allocates a whole page for like 7 bytes
#[global_allocator]
pub static mut ALLOCATOR: PhysicalPageAllocator<HEAP_SIZE> = PhysicalPageAllocator::new();

impl<const PAGES: usize> PhysicalPageAllocator<PAGES>
where
    [u8; pages_subdivide(PAGES)]: Sized,
{
    pub const fn new() -> Self {
        Self {
            start: 0,
            descriptors: [PageFlags::Free.val(); pages_subdivide(PAGES)],
        }
    }

    /// Initializes based on _heap_start global constant.
    ///
    /// # Safety
    /// Only safe to call from a single allocator, otherwise multiple allocators
    /// will have the same base address.
    pub unsafe fn default_init(&mut self) {
        // mask out bottom bits to align to page
        let val = {
            let heap_start = &__heap_start as *const _ as usize;
            if heap_start % PAGE_SIZE == 0 {
                heap_start
            } else {
                let new_val = (heap_start + PAGE_SIZE) & !(PAGE_SIZE - 1);
                printk!(
                    "Original heap_start is not page aligned, rounding 0x{:x} -> 0x{:x}",
                    heap_start,
                    new_val
                );
                new_val
            }
        };
        self.init(val);
    }

    /// Initializes with the given start value.
    ///
    /// # Safety
    /// Only safe to call if the start address is valid.
    pub unsafe fn init(&mut self, start: usize) {
        self.start = start;
    }

    pub const fn get_base(&self) -> usize {
        self.start
    }

    fn assert_init(&self) {
        if self.start == 0 {
            panic!("allocator is uninitialized!");
        }
    }

    /// Try to allocate the contiguous region of pages, returning the pointer to the region if possible.
    pub fn try_allocate(&mut self, size: usize) -> Option<*mut u8> {
        self.assert_init();
        let pages = size_to_pages(size);
        assert!(pages > 0, "Can't make an empty allocation");
        let mut begin_index = 0;
        let mut matching = 0;
        let begin_index: Option<usize> = 'block: {
            for (i, entry) in self.descriptors.iter().enumerate() {
                // printk!(
                //     "Trying entry #{} begin: {} matching: {} needed: {}",
                //     i,
                //     begin_index,
                //     matching,
                //     pages
                // );
                let flags = *entry;
                if flags & PageFlags::Free.val() != 0 {
                    matching += 1;
                } else {
                    matching = 0;
                    begin_index = i + 1;
                }
                if matching >= pages {
                    // note: ra marks this as an error but it's actually fine
                    // (see rust-analyzer#4747)
                    // due to #![feature(label_break_value)]
                    //printk!("Success!");
                    break 'block Some(begin_index);
                }
            }
            None
        };
        if let Some(begin_index) = begin_index {
            //printk!("begin_index is found");
            // Mark all descriptors as taken and return value.
            for descriptor in self.descriptors[begin_index..=begin_index + pages].iter_mut() {
                *descriptor = (*descriptor) & !PageFlags::Free.val() | PageFlags::Taken.val();
            }
            Some((self.start + (begin_index * PAGE_SIZE)) as _)
        } else {
            None
        }
    }

    /// Same as try_allocate, but also zeroes the range
    pub fn try_zallocate(&mut self, size: usize) -> Option<*mut u8> {
        let pages = match self.try_allocate(size) {
            Some(pointer) => pointer,
            None => return None,
        };
        let slice = unsafe { core::slice::from_raw_parts_mut(pages, size) };
        for thing in slice.iter_mut() {
            *thing = 0;
        }
        Some(pages)
    }

    /// Prints the page allocation table as a 32xN square
    pub fn print_page_allocation_table(&self) {
        self.assert_init();
        for chunk in self.descriptors.chunks(32) {
            for descriptor in chunk {
                if (*descriptor) & PageFlags::Free.val() != 0 {
                    print!(".");
                }
                if (*descriptor) & PageFlags::Taken.val() != 0 {
                    print!("X");
                }
            }
            println!();
        }
        println!();
        println!(". = free, X = taken");
    }

    /// Deallocates the given region of pages.
    pub fn deallocate(&mut self, addr: *mut u8, size: usize) {
        self.assert_init();
        let addr = addr as usize - self.start;
        let pages = size_to_pages(size);
        let begin_index = addr / PAGE_SIZE;
        for descriptor in self.descriptors[begin_index..=begin_index + pages].iter_mut() {
            *descriptor = (*descriptor) & !PageFlags::Taken.val() | PageFlags::Free.val();
        }
    }

    /// Gets the number of used pages.
    pub fn used(&self) -> usize {
        self.assert_init();
        self.descriptors
            .iter()
            .filter(|&&d| d & PageFlags::Taken.val() != 0)
            .count()
    }

    /// Gets the total number of pages.
    pub const fn total(&self) -> usize {
        self.descriptors.len()
    }
}

const fn size_to_pages(size: usize) -> usize {
    let u = size / PAGE_SIZE;
    if size % PAGE_SIZE > 0 {
        u + 1
    } else {
        u
    }
}

unsafe impl<const PAGES: usize> GlobalAlloc for PhysicalPageAllocator<PAGES>
where
    [u8; pages_subdivide(PAGES)]: Sized,
{
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        //printk!("allocating size = {}", layout.size());
        ALLOCATOR
            .try_allocate(layout.size())
            .expect("Failed to make allocation for global allocator") as _
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        //printk!("addr = {:?} size = {}", ptr, layout.size());
        ALLOCATOR.deallocate(ptr as _, layout.size());
    }
}

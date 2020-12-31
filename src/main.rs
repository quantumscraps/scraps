#![feature(asm)]
#![feature(global_asm)]
#![feature(const_fn)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)] // NOTE: const_evaluatable_unchecked isn't a thing !!
#![feature(const_panic)]
#![feature(default_alloc_error_handler)]
#![feature(label_break_value)]
#![feature(naked_functions)]
#![allow(incomplete_features)]
#![deny(missing_docs)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_const_for_fn)]
#![deny(clippy::missing_safety_doc)]
#![warn(clippy::all)]
#![no_main]
#![no_std]

//! Scraps of an operating system

extern crate alloc;

mod arch;
mod bsp;
mod cpu;
mod driver_interfaces;
mod drivers;
mod memory;
mod mmu;
mod panic;
mod physical_page_allocator;
mod print;
mod time;
//mod util;

use alloc::string::String;
use alloc::vec::Vec;
use driver_interfaces::*;
use fdt_rs::base::DevTree;
use fdt_rs::index::{DevTreeIndex, DevTreeIndexItem};
use fdt_rs::prelude::*;
use mmu::{enable_paging, enable_smode, map_gigapage, Sv39PageTable};
use physical_page_allocator::{ALLOCATOR, PAGE_SIZE};

/// Creates a static ref to a linker variable
#[macro_export]
macro_rules! link_var {
    ($visi:vis $name:ident) => {
        extern "C" { $visi static $name: core::ffi::c_void; }
    };
    ($($toks:tt),+) => {
        $(link_var!($tt);)+
    }
}

/// Looks up a DTB entry by path
pub fn lookup_dtb_entry<'dt>(
    dtb: &'dt DevTreeIndex,
    path: &str,
) -> Option<DevTreeIndexItem<'dt, 'dt, 'dt>> {
    // remove root
    let path = path.trim_start_matches('/');
    let mut current_node = dtb.root();
    let mut prop = None;
    let mut consumed = 0;
    let mut len = 0;

    for component in path.split('/') {
        len += 1;
        for child in current_node.children() {
            if child.name() == Ok(component) {
                current_node = child;
                consumed += 1;
                continue;
            }
        }
        // if we are here there are no matching children
        // so check props instead
        for prop2 in current_node.props() {
            if prop2.name() == Ok(component) {
                prop = Some(prop2);
                consumed += 1;
                // properties are leaves, break
                break;
            }
        }
    }

    // Check if we consumed all components
    if consumed == len {
        Some(if let Some(prop) = prop {
            DevTreeIndexItem::Prop(prop)
        } else {
            DevTreeIndexItem::Node(current_node)
        })
    } else {
        None
    }
}

/// Used to test if paging worked
const PAGING_TEST: usize = 0x1010101010101010;

/// The early entry point for initializing the OS.
/// Paging, DTB, etc. are setup here.
///
/// # Safety
/// Safe only to call from [setup_environment].
///
/// [setup_environment]: memory::setup_environment
#[no_mangle]
pub unsafe extern "C" fn kinit(dtb_addr: *mut u8) -> ! {
    //mmu::init();
    bsp::UNSAFE_UART.init();
    let v = 12;
    printk!("dtb_addr = {:?}", dtb_addr);
    // init allocator
    ALLOCATOR.default_init();
    mmu::init();
    let r = DevTree::read_totalsize(core::slice::from_raw_parts(
        dtb_addr as *const _,
        DevTree::MIN_HEADER_SIZE,
    ))
    .and_then(|size| DevTree::new(core::slice::from_raw_parts(dtb_addr as *const _, size)));
    if let Ok(r) = r {
        printk!("Success reading DTB");
        if let Ok(layout) = DevTreeIndex::get_layout(&r) {
            printk!("Got DTB index layout");
            let mut ivec = alloc::vec![0u8; layout.size() + layout.align()];
            if let Ok(index) = DevTreeIndex::new(r, ivec.as_mut_slice()) {
                printk!("Created index");
                if let Some(DevTreeIndexItem::Prop(prop)) =
                    lookup_dtb_entry(&index, "/chosen/bootargs")
                {
                    printk!("cmdline = {}", prop.str().unwrap());
                }
            }
        }
        /*for item in r.struct_items() {
            if let Ok(name) = item.name() {
                printk!("Name = {}", name);
            }
        }*/
        for entry in r.reserved_entries() {
            printk!("reserved: {:?}, {:?}", entry.address, entry.size);
        }
    } else if let Err(e) = r {
        printk!("Failed to read dtb error = {:?}", e);
    }
    printk!("Address of some stack variable is {:?}", (&v as *const _));
    printk!(
        "Timer Accuracy: {} ns",
        time::time_counter().accuracy().as_nanos()
    );
    printk!("PPE = {:?}", ALLOCATOR.get_base() as *const i8);
    printk!("Allocating 35 pages...");
    let allocation = ALLOCATOR.try_allocate(35 * PAGE_SIZE);
    if let Some(allocation) = allocation {
        printk!("Success! Allocation address = {:?}", allocation);
    } else {
        printk!("Failure...");
    }
    ALLOCATOR.print_page_allocation_table();
    if let Some(allocation) = allocation {
        printk!("Freeing allocation...");
        ALLOCATOR.deallocate(allocation, 35 * PAGE_SIZE);
        ALLOCATOR.print_page_allocation_table();
    }
    // Allocate and reserve a vec
    printk!("Allocating a vec<string> and reserving 37 items, then pushing a bunch of strings...");
    let mut v: Vec<String> = Vec::with_capacity(37);
    for _ in 0..37 {
        v.push(String::from("testabc"));
    }
    ALLOCATOR.print_page_allocation_table();
    printk!("Dropping vec..");
    drop(v);
    ALLOCATOR.print_page_allocation_table();
    //printk!("Heap size = {}", _heap_size);
    // loop {
    //     printk!("Hello, World!");
    //     time::time_counter().wait_for(Duration::from_secs(1));
    // }
    //cpu::wait_forever()
    printk!("Allocating root table...");
    #[cfg(target_arch = "riscv64")]
    {
        printk!("Enabling S-mode...");
        enable_smode(kinit2 as usize);
    }
    #[cfg(not(target_arch = "riscv64"))]
    {
        printk!("Unsupported for non-RISCV platforms, for now");
    }
    cpu::wait_forever()
}

unsafe fn kinit2() -> ! {
    let root_table_addr = ALLOCATOR
        .try_allocate(PAGE_SIZE)
        .expect("Couldn't allocate page!");
    printk!("Root table addr = {}", root_table_addr as usize);
    printk!(
        "root_addr % PAGE_SIZE = {}",
        root_table_addr as usize % PAGE_SIZE
    );
    // Rust is smart :)
    let root_table: &mut Sv39PageTable = &mut *(root_table_addr as *mut _);
    root_table.init();

    link_var!(__kern_start);
    let kern_addr = &__kern_start as *const _ as u64;
    let onegig = 0x40000000u64;
    let kern_addr_rounded = kern_addr & !(onegig - 1);
    printk!("Mapping UART addr = 0x{:x}", 0x1000_0000);
    map_gigapage(root_table, 0x1000_0000, 0x1000_0000);
    printk!("Mapping kern_addr (rounded) ~= 0x{:x}", kern_addr_rounded);
    // 1) identity map
    map_gigapage(root_table, kern_addr, kern_addr);
    let hh_base = 0x2000000000u64;
    printk!("Mapping hh_addr ~= 0x{:x}", hh_base);
    // 2) let the higher half be something like 0x2000000000
    map_gigapage(root_table, hh_base, kern_addr);
    root_table.print();
    // calculate addresses for identity and hh of PAGING_TEST
    let paging_test_identity_addr = &PAGING_TEST as *const _ as u64;
    // round kern_addr to 1g
    let paging_test_hh_addr = paging_test_identity_addr - kern_addr_rounded + hh_base;
    // enable paging and jump to higher half too
    let kinit2_hh_addr = (kinit2 as u64) - kern_addr_rounded + hh_base;
    enable_paging(root_table);
    let satp_value: u64;
    asm!("csrr {0}, satp", out(reg) satp_value);
    printk!("Read SATP = {:064b}", satp_value);
    //print!("yess");
    printk!("It worked?");
    // check values
    printk!(
        "PAGING_TEST from identity map: {:x}",
        *(paging_test_identity_addr as *const usize)
    );
    printk!("*hh_base = {}", *(hh_base as *const usize));
    printk!(
        "PAGING_TEST from hh map:       {:x}",
        *(paging_test_hh_addr as *const usize)
    );
    cpu::wait_forever()
}

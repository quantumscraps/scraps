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
#![no_main]
#![no_std]

//! Scraps of an operating system

extern crate alloc;

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

use crate::time::TimeCounter;
use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;
use driver_interfaces::*;
use fdt_rs::base::DevTree;
use fdt_rs::index::{DevTreeIndex, DevTreeIndexItem};
use fdt_rs::prelude::*;
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

/// The early entry point for initializing the OS.
/// Paging, DTB, etc. are setup here.
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
    loop {
        printk!("Hello, World!");
        time::time_counter().wait_for(Duration::from_secs(1));
    }
    //cpu::wait_forever()
}

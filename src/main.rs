#![feature(asm)]
#![feature(global_asm)]
#![feature(const_fn)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)] // NOTE: const_evalutable_unchecked isn't a thing !!
#![feature(const_panic)]
#![feature(default_alloc_error_handler)]
#![feature(label_break_value)]
#![feature(naked_functions)]
#![allow(incomplete_features)]
#![no_main]
#![no_std]

extern crate alloc;

mod bsp;
mod cpu;
mod driver_interfaces;
mod drivers;
mod panic;
mod physical_page_allocator;
mod print;
mod time;
mod util;
mod mmu;

use crate::time::TimeCounter;
use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;
use driver_interfaces::*;
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

// still unsafe because mutable statics are unsafe !!
// we need a mutex eventually
#[no_mangle]
pub unsafe extern "C" fn kinit(dtb_addr: *mut i8) -> ! {
    mmu::init();
    bsp::UNSAFE_UART.init();
    let v = 12;
    printk!("dtb_addr = {:?}", dtb_addr);
    let r = dtb::Reader::read_from_address(dtb_addr as _);
    if let Ok(r) = r {
        /*for item in r.struct_items() {
            if let Ok(name) = item.name() {
                printk!("Name = {}", name);
            }
        }*/
        for entry in r.reserved_mem_entries() {
            printk!("reserved: {:?}, {:?}", entry.address, entry.size);
        }
        /*if let Some((cmdline, _)) = r.struct_items().path_struct_items("/chosen/bootargs").next() {
            println!("[cmdline] {}", cmdline.value_str().unwrap());
        }*/

    } else if let Err(e) = r {
        printk!("Failed to read dtb error = {:?}", e);
    }
    printk!("Address of some stack variable is {:?}", (&v as *const _));
    printk!(
        "Timer Accuracy: {} ns",
        time::time_counter().accuracy().as_nanos()
    );
    // init allocator
    ALLOCATOR.lock().default_init();
    printk!("PPE = {:?}", ALLOCATOR.lock().get_base() as *const i8);
    printk!("Allocating 35 pages...");
    let allocation = ALLOCATOR.lock().try_allocate(35 * PAGE_SIZE);
    if let Some(allocation) = allocation {
        printk!("Success! Allocation address = {:?}", allocation);
    } else {
        printk!("Failure...");
    }
    ALLOCATOR.lock().print_page_allocation_table();
    if let Some(allocation) = allocation {
        printk!("Freeing allocation...");
        ALLOCATOR.lock().deallocate(allocation, 35 * PAGE_SIZE);
        ALLOCATOR.lock().print_page_allocation_table();
    }
    // Allocate and reserve a vec
    printk!("Allocating a vec<string> and reserving 37 items, then pushing a bunch of strings...");
    let mut v: Vec<String> = Vec::with_capacity(37);
    for _ in 0..37 {
        v.push(String::from("testabc"));
    }
    ALLOCATOR.lock().print_page_allocation_table();
    printk!("Dropping vec..");
    drop(v);
    ALLOCATOR.lock().print_page_allocation_table();
    //printk!("Heap size = {}", _heap_size);
    loop {
        printk!("Hello, World!");
        time::time_counter().wait_for(Duration::from_secs(1));
    }
    //cpu::wait_forever()
}

#![feature(asm)]
#![feature(alloc_prelude)]
#![feature(global_asm)]
#![feature(const_fn)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)] // NOTE: const_evaluatable_unchecked isn't a thing !!
#![feature(const_in_array_repeat_expressions)]
#![feature(const_panic)]
#![feature(const_ptr_offset)]
#![feature(const_size_of_val)]
#![feature(default_alloc_error_handler)]
#![feature(label_break_value)]
#![feature(layout_for_ptr)]
#![feature(naked_functions)]
#![feature(never_type)]
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
mod process;
mod time;
mod util;

use arch::mmu::{SvTable, XWRPermissions, __root_page_table};
use driver_interfaces::{Console, UartConsole};
use process::Process;
use util::UnsafeMutex;

/// Creates a static ref to a linker variable
#[macro_export]
macro_rules! link_var {
    ($visi:vis $name:ident) => {
        extern "C" { $visi static $name: core::ffi::c_void; }
    };
    ($($toks:tt),+) => {
        $(link_var!($toks);)+
    }
}

/// Used to test if paging worked
const PAGING_TEST: usize = 0x1010101010101010;

/// Default output device
static STDOUT: UnsafeMutex<Option<UartConsole>> = UnsafeMutex::new(None);

/// Processes!
static PROCESSES: UnsafeMutex<[Option<Process>; 4]> = UnsafeMutex::new([None; 4]);

/// The early entry point for initializing the OS.
/// Paging, DTB, etc. are setup here.
///
/// # Safety
/// Safe only to call from [setup_environment].
///
/// [setup_environment]: memory::setup_environment
#[no_mangle]
pub extern "C" fn kinit(dtb_addr: *mut u8) {
    // unmap old kernel base
    // unsafe {
    //     __root_page_table.unmap_gigapage(old_kern_start);
    //     asm!("sfence.vma");
    // }

    // unsafe {
    //     *STDOUT.get_mut() = Some(drivers::known_good_uart());
    //     // map uart
    //     let addr = STDOUT.get_mut().as_mut().unwrap().base_address();
    //     __root_page_table.map_page(
    //         addr,
    //         addr,
    //         XWRPermissions::Read | XWRPermissions::Write | XWRPermissions::Execute,
    //     );
    // }

    // unsafe {
    //     // setup allocator
    //     // ALLOCATOR.default_init();
    //     // setup mmu
    //     // arch::mmu::init();
    // };

    printk!("dtb_addr = {:x}", dtb_addr as usize);
    printk!("Initialized ppa and mmu");

    printk!("Stack is broken, right?");
    printk!("HAHA NO ITS NOT!!!!!!");

    // Lets try to access some invalid memory
    unsafe { (0 as *const usize).read_volatile() };

    cpu::wait_forever()
}

#![feature(asm)]
#![feature(alloc_prelude)]
#![feature(global_asm)]
#![feature(const_fn)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)] // NOTE: const_evaluatable_unchecked isn't a thing !!
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
mod time;
mod util;

use driver_interfaces::UartConsole;
use fdt_rs::base::DevTree;
use util::{HeaplessResult, UnsafeMutex};

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

/// Used to test if paging worked
const PAGING_TEST: usize = 0x1010101010101010;

/// Default output device
static STDOUT: UnsafeMutex<Option<UartConsole>> = UnsafeMutex::new(None);

/// The early entry point for initializing the OS.
/// Paging, DTB, etc. are setup here.
///
/// # Safety
/// Safe only to call from [setup_environment].
///
/// [setup_environment]: memory::setup_environment
#[no_mangle]
#[allow(improper_ctypes_definitions)] // We only use extern "C" for calling convention
pub extern "C" fn kinit(dtb_addr: *mut u8) -> HeaplessResult<!> {
    let dtb = unsafe {
        let size = DevTree::read_totalsize(core::slice::from_raw_parts(
            dtb_addr as *const _,
            DevTree::MIN_HEADER_SIZE,
        ))?;
        DevTree::new(core::slice::from_raw_parts(dtb_addr as *const _, size))?
    };
    drivers::detect_stdout(&dtb)?;

    cpu::wait_forever()
}

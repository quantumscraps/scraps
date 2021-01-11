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

use alloc::prelude::v1::*;
use driver_interfaces::Console;
use fdt_rs::base::DevTree;
use fdt_rs::index::{DevTreeIndex, DevTreeIndexItem};
use fdt_rs::prelude::*;
use util::UnsafeMutex;

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

/// Default output device
static STDOUT: UnsafeMutex<Option<Box<dyn Console>>> = UnsafeMutex::new(None);

/// The early entry point for initializing the OS.
/// Paging, DTB, etc. are setup here.
///
/// # Safety
/// Safe only to call from [setup_environment].
///
/// [setup_environment]: memory::setup_environment
#[no_mangle]
pub unsafe extern "C" fn kinit(dtb_addr: *mut u8) -> ! {
    cpu::wait_forever()
}

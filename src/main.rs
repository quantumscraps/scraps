#![feature(asm)]
#![feature(duration_zero)]
#![feature(global_asm)]
#![no_main]
#![no_std]

mod bsp;
mod cpu;
mod driver_interfaces;
mod drivers;
mod lock;
mod memory;
mod mutex;
mod panic;
mod print;
mod time;
mod util;

use crate::time::TimeCounter;
use core::time::Duration;
use driver_interfaces::*;

// still unsafe because mutable statics are unsafe !!
// we need a mutex eventually
#[no_mangle]
pub unsafe extern "C" fn kernel_init(dtb_addr: *mut i8) -> ! {
    bsp::UART.get().init();
    let v = 12;
    printk!("dtb_addr = {:?}", dtb_addr);
    let r = dtb::Reader::read_from_address(dtb_addr as _);
    if let Ok(r) = r {
        for item in r.struct_items() {
            if let Ok(name) = item.name() {
                printk!("Name = {}", name);
            }
        }
    } else if let Err(e) = r {
        printk!("Failed to read dtb error = {:?}", e);
    }
    // for testing ig
    let lock = lock::Lock::new();
    printk!("Address of some stack variable is {:?}", (&v as *const _));
    printk!(
        "Timer Accuracy: {} ns",
        time::time_counter().accuracy().as_nanos()
    );
    loop {
        printk!("Hello, World!");
        time::time_counter().wait_for(Duration::from_secs(1));
    }
    //cpu::wait_forever()
}

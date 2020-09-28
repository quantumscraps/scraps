#![feature(asm)]
#![feature(global_asm)]
#![no_main]
#![no_std]

mod bsp;
mod cpu;
mod memory;
mod panic;
mod print;
mod util;
mod time;
mod drivers;
mod driver_interfaces;

use driver_interfaces::*;
use core::time::Duration;
use crate::time::TimeCounter;


// still unsafe because mutable statics are unsafe !!
// we need a mutex eventually
#[no_mangle]
pub unsafe extern "C" fn kernel_init() -> ! {
    bsp::UART.init();
    let v = 12;
    printk!("Address of some stack variable is {:?}", (&v as *const _));
    printk!("Timer Accuracy: {} ns", time::time_counter().accuracy().as_nanos());
    loop {
        printk!("Hello, World!");
        time::time_counter().wait_for(Duration::from_secs(1));
    }
    //cpu::wait_forever()
}

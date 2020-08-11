#![feature(asm)]
#![feature(global_asm)]
#![no_main]
#![no_std]

mod bsp;
mod cpu;
mod memory;
mod panic;
mod util;
mod drivers;
mod driver_interfaces;

use core::fmt::Write;
use driver_interfaces::*;

#[no_mangle]
pub unsafe extern "C" fn kernel_init() -> ! {
    bsp::UART.init();
    let _ = writeln!(bsp::UART, "Hello, World!");
    cpu::wait_forever()
}

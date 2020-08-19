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
mod drivers;
mod driver_interfaces;

use driver_interfaces::*;

#[no_mangle]
pub unsafe extern "C" fn kernel_init() -> ! {
    bsp::UART.init();
    let _ = println!("[{}] Hello, World!", 0);
    cpu::wait_forever()
}

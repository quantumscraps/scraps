#![feature(asm)]
#![feature(global_asm)]
#![no_main]
#![no_std]

mod bsp;
mod cpu;
mod memory;
mod panic;

#[no_mangle]
pub unsafe extern "C" fn kernel_init() -> ! {
    cpu::wait_forever()
}

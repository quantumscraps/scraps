use crate::{cpu, drivers, println2, util::HeaplessResult};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // let stdout = unsafe { STDOUT.get_mut() };
    let mut uart = drivers::known_good_uart();

    println2!(Some(&mut uart), "[!] Kernel Panic: {}", _info);
    cpu::wait_forever()
}

#[no_mangle]
#[allow(improper_ctypes_definitions)] // We only use extern "C" for calling convention
pub extern "C" fn print_backtrace_from_kinit(err: HeaplessResult<!>) {
    let err = match err {
        Ok(_) => unreachable!(),
        Err(e) => e,
    };
    // Stdout may not be initialized at this point, so try a known good address...
    let mut uart = drivers::known_good_uart();

    println2!(Some(&mut uart), "[!!] kinit failed: {}", err);
}

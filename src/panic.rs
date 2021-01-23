use crate::{cpu, drivers, println2};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // let stdout = unsafe { STDOUT.get_mut() };
    let mut uart = drivers::known_good_uart();

    println2!(Some(&mut uart), "[!] Kernel Panic: {}", _info);
    cpu::wait_forever()
}

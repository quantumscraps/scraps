use crate::{cpu, println2, STDOUT};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let stdout = unsafe { STDOUT.get_mut() };
    println2!(stdout, "[!] Kernel Panic: {}", _info);
    cpu::wait_forever()
}

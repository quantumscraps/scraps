use crate::cpu;
use crate::panic_println;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    panic_println!("[!] Kernel Panic: {}", _info);
    cpu::wait_forever()
}

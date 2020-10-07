use crate::cpu;
use crate::println;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    cpu::wait_forever()
}

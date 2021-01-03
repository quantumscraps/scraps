use crate::drivers::pl011::PL011;
use cortex_a::regs::*;
use spin::Mutex;

pub static UART: Mutex<PL011> = Mutex::new(PL011::new());
//pub static mut UNSAFE_UART: PL011 = PL011::new();

pub fn mmio_base() -> usize {
    match (MIDR_EL1.get() >> 4) & 0xFFF {
        0xB76 => 0x2000_0000,
        0xC07 => 0x3F00_0000,
        0xD03 => 0x3F00_0000,
        0xD08 => 0xFE00_0000,
        _ => 0x2000_0000,
    }
}

pub const HEAP_SIZE: usize = 0x100000; // PAGE_SIZE * 1048576; // 1m allocations
pub const PAGE_SIZE: usize = 0x1000;

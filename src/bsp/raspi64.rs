use crate::drivers::pl011::PL011;

pub static mut UART: PL011 = PL011::new();

pub fn mmio_base() -> usize {
    unsafe {
        let midr_el1: usize;
        asm!("mrs x5, midr_el1",
            out("x5") midr_el1,
        );
        match (midr_el1 >> 4) & 0xFFF {
        0xB76 => 0x2000_0000,
        0xC07 => 0x3F00_0000,
        0xD03 => 0x3F00_0000,
        0xD08 => 0xFE00_0000,
        _     => 0x2000_0000
        }
    }
}

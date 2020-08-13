use crate::drivers::pl011::PL011;

pub static mut UART: PL011 = PL011::new();

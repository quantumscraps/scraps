use crate::drivers::ns16550a::NS16550A;

pub static mut UART: NS16550A = NS16550A::new(0x1000_0000);

use crate::drivers::ns16550a::NS16550A;

pub static mut UART: NS16550A = NS16550A::new(0x1000_0000);

pub const NANOS_PER_TICK: u64 = 1;
pub const HAS_RDTIME: bool = false;
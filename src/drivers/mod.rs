use crate::driver_interfaces::{Console, UartConsole};

use self::ns16550a::NS16550A;

#[cfg(feature = "bsp_riscvirt")]
pub mod ns16550a;

#[cfg(feature = "bsp_raspi64")]
pub mod pl011;

/// Gets a known good UART.
pub fn known_good_uart() -> UartConsole {
    #[cfg(target_arch = "riscv64")]
    let mut uart = UartConsole::NS16550A(unsafe { NS16550A::new(0x1000_0000) });
    #[cfg(target_arch = "aarch64")]
    let mut uart = UartConsole::PL011(unsafe { PL011::new(0x4000_0000) });
    uart.init();
    uart
}

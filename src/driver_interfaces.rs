use core::fmt::{Debug, Write};

use crate::drivers::ns16550a::NS16550A;

/// A UART that can be written to.
pub trait Uart: Write {
    /// Initializes the UART.
    /// This function must be safe to call multiple times.
    fn init(&mut self);
    /// Gets an input byte, if any.
    fn get(&mut self) -> Option<u8>;
    /// Writes a byte to the output.
    fn put(&mut self, value: u8);
}

/// Some kind of console that can be used for output.
pub trait Console: Write + Debug {
    /// Initializes this console for output.
    fn init(&mut self);

    /// Gets the base address of this console.
    fn base_address(&self) -> usize;
}

#[derive(Debug, Clone)]
/// Statically sized enum that represents all UARTs.
pub enum UartConsole {
    #[cfg(feature = "bsp_riscvirt")]
    NS16550A(NS16550A),
    #[cfg(feature = "bsp_raspi64")]
    PL011(PL011),
}

impl UartConsole {
    fn console(&self) -> &dyn Console {
        match self {
            #[cfg(feature = "bsp_riscvirt")]
            Self::NS16550A(ref uart) => uart,
            #[cfg(feature = "bsp_raspi64")]
            Self::PL011(ref uart) => uart,
        }
    }

    fn console_mut(&mut self) -> &mut dyn Console {
        match self {
            #[cfg(feature = "bsp_riscvirt")]
            Self::NS16550A(ref mut uart) => uart,
            #[cfg(feature = "bsp_raspi64")]
            Self::PL011(ref mut uart) => uart,
        }
    }
}

impl Write for UartConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.console_mut().write_str(s)
    }
}

impl Console for UartConsole {
    fn init(&mut self) {
        self.console_mut().init()
    }

    fn base_address(&self) -> usize {
        self.console().base_address()
    }
}

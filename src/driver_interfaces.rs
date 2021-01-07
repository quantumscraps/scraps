use core::fmt::{Debug, Write};

use fdt_rs::index::DevTreeIndexNode;

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

    /// Creates a console from the given device tree node.
    /// Can return `None` if it was not possible to create
    /// this kind of console from the given node.
    fn from_dtb(dtb: &DevTreeIndexNode) -> Option<Self>
    where
        Self: Sized;
}

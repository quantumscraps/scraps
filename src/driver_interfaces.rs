use core::fmt::Write;

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

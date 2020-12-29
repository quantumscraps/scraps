use core::fmt::Write;

/// A UART that can be written to.
pub trait Uart: Write {
    /// (Unsafely) initializes the UART.
    ///
    /// # Safety
    /// Only safe to call once.
    unsafe fn init(&mut self);
    /// Gets an input byte, if any.
    fn get(&mut self) -> Option<u8>;
    /// Writes a byte to the output.
    fn put(&mut self, value: u8);
}

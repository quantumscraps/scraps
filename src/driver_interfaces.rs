use core::fmt::Write;

pub trait Uart: Write {
    unsafe fn init(&mut self);
    fn get(&mut self) -> Option<u8>;
    fn put(&mut self, value: u8);
}

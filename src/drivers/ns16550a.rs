use crate::driver_interfaces::Uart;
use crate::util;

pub struct NS16550A {
    base_address: usize,
}

impl NS16550A {
    pub const fn new(base_address: usize) -> Self {
        Self { base_address }
    }
}

impl core::fmt::Write for NS16550A {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        for c in s.bytes() {
            self.put(c);
        }
        Ok(())
    }
}

impl Uart for NS16550A {
    unsafe fn init(&mut self) {
        let base = self.base_address;
        // Word length = 3
        let lcr = 0b11;
        util::sb(base + 3, lcr);
        // Enable FIFO
        util::sb(base + 2, 1);
        // Enable receiver buffer interrupts
        util::sb(base + 1, 1);
        // Set the divisor to 2400 baud or whatever
        // doesn't really do anything in qemu
        // open the DLB to set divisor
        util::sb(base + 3, lcr | 1 << 7);
        let divisor = 592u16;
        let divisor_least = (divisor & 0xff) as u8;
        let divisor_most = (divisor >> 8 & 0xff) as u8;
        util::sb(base, divisor_least);
        util::sb(base + 1, divisor_most);
        // close the DLB for normal use
        util::sb(base + 3, lcr);
    }

    fn get(&mut self) -> Option<u8> {
        unsafe {
            if util::lb(self.base_address + 5) & 1 == 0{
                None
            } else {
                Some(util::lb(self.base_address))
            }
        }
    }

    fn put(&mut self, value: u8) {
        unsafe {
            util::sb(self.base_address, value);
        }
    }
}

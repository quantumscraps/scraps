use crate::driver_interfaces::Uart;
use crate::util::mmio;
use crate::{bsp, cpu};
mod uart {
    pub const DR: usize = 0x00201000;
    pub const FR: usize = 0x00201018;
    pub const IBRD: usize = 0x00201024;
    pub const FBRD: usize = 0x00201028;
    pub const LCRH: usize = 0x0020102C;
    pub const CR: usize = 0x00201030;
    pub const ICR: usize = 0x00201044;
}
mod gpio {
    pub const GPFSEL1: usize = 0x00200004;
    pub const GPPUD: usize = 0x00200094;
    pub const GPPUDCLK0: usize = 0x00200098;
}
pub struct PL011 {
    base: usize,
}

impl PL011 {
    pub const fn new() -> Self {
        Self { base: 0x2000_0000 }
    }
}

impl core::fmt::Write for PL011 {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        for c in s.bytes() {
            self.put(c);
        }
        Ok(())
    }
}

impl Uart for PL011 {
    unsafe fn init(&mut self) {
        self.base = bsp::mmio_base();
        // get GPFSEL1 and map pins 14 and 15 to TX and RX
        let mut r: u32 = mmio::ld(self.base + gpio::GPFSEL1);
        r &= !((7 << 12) | (7 << 15));
        r |= (4 << 12) | (4 << 15);
        mmio::sd(self.base + gpio::GPFSEL1, r);
        // enable pins 14 and 15
        mmio::sd(self.base + gpio::GPPUD, 0);
        cpu::spin_for_cycles(150);
        mmio::sd(self.base + gpio::GPPUDCLK0, (1 << 14) | (1 << 15));
        cpu::spin_for_cycles(150);
        // flush GPIO setup
        mmio::sd(self.base + gpio::GPPUDCLK0, 0);

        // turn off UART temporarily with CR (Control Register)
        mmio::sd(self.base + uart::CR, 0);
        // clear all interrupts with ICR (Interrupt Clear Register)
        mmio::sd(self.base + uart::ICR, 0x7FF);
        // set IBRD (Integer Baud Rate Divisor) to 13
        // because (48MHz/16)/230400 = 13.02083, margin of error is acceptable
        mmio::sd(self.base + uart::IBRD, 13);
        // set FBRD (Fractional Baud Rate Divisor) to 1
        // because 0.02083*64 = 1.3312, rounded to 1
        mmio::sd(self.base + uart::FBRD, 1);
        // set LCRH to 8 bit chars and enable FIFO
        mmio::sd(self.base + uart::LCRH, 0b11 << 5 | 0b1 << 4);
        // set CR to enable UART, TX, and RX
        mmio::sd(self.base + uart::CR, 0b1 << 9 | 0b1 << 8 | 0b1);
    }

    fn get(&mut self) -> Option<u8> {
        unsafe {
            // if character is not available return None
            // if it is return character
            if (mmio::ld(self.base + uart::FR) & 0x20) == 0x20 {
                None
            } else {
                Some(mmio::ld(self.base + uart::DR) as u8)
            }
        }
    }

    fn put(&mut self, value: u8) {
        unsafe {
            // if TX is busy wait till it's not
            loop {
                if !((mmio::ld(self.base + uart::FR) & 0x20) == 0x20) {
                    break;
                }
                cpu::nop();
            }
            mmio::sd(self.base + uart::DR, value as u32);
        }
    }
}

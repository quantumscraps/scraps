use crate::driver_interfaces::Uart;
use crate::util::mmio;
use crate::cpu;

pub struct PL011 {
    base: usize
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
        let midr_el1: usize;
        asm!("mrs x5, midr_el1",
             out("x5") midr_el1,
        );
        self.base = match ((midr_el1 >> 4) & 0xFFF) {
           0xB76 => 0x2000_0000,
           0xC07 => 0x3F00_0000,
           0xD03 => 0x3F00_0000,
           0xD08 => 0xFE00_0000,
           _     => 0x2000_0000
        };
        // turn off UART temporarily with CR (Control Register)
        mmio::sd(self.base + 0x0020_1030, 0);
        // clear all interrupts with ICR (Interrupt Clear Register)
        mmio::sd(self.base + 0x0020_1044, 0x7FF);
        // set IBRD (Integer Baud Rate Divisor) to 13
        // because (48MHz/16)/230400 = 13.02083, margin of error is acceptable
        mmio::sd(self.base + 0x0020_1024, 13);
        // set FBRD (Fractional Baud Rate Divisor) to 1
        // because 0.02083*64 = 1.3312, rounded to 1
        mmio::sd(self.base + 0x0020_1028, 1);
        // set LCRH to 8 bit chars and enable FIFO
        mmio::sd(self.base + 0x0020_102C, 0b11<<5 | 0b1<<4);
        // set CR to enable UART, TX, and RX
        mmio::sd(self.base + 0x0020_1030, 0b1 << 9 | 0b1 << 8 | 0b1);
    }

    fn get(&mut self) -> Option<u8> {
        unsafe {
            // if character is not available return None
            // if it is return character
            if (mmio::ld(self.base + 0x0020_1018) & 0x10) != 0 {
                None
            } else {
                Some(mmio::ld(self.base + 0x0020_1000) as u8)
            }
        }
    }

    fn put(&mut self, value: u8) {
        unsafe {
            // if TX is busy wait till it's not
            // then send character
            while (mmio::ld(self.base + 0x0020_1018) & 0x20) != 0 {
                cpu::nop();
            }
            mmio::sd(self.base + 0x0020_1000, value as u32);
        }
    }
}

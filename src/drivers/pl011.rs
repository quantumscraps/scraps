use crate::driver_interfaces::*;
use crate::{bsp, cpu};
use register::{mmio::*, register_bitfields, register_structs};
register_bitfields! {
    // 32 bit wide registers
    u32,
    //GPIO Function Select 1
    GPFSEL1 [
        FSEL15 OFFSET(15) NUMBITS(0b11) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100 // PL011 RX
        ],
        FSEL14 OFFSET(12) NUMBITS(0b11) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100 // PL011 TX
        ]
    ],
    // GPIO Pin Pull-up/down Enable
    GPPUD [
        // GPIO Pin Pull-up/down
        PUD OFFSET(0) NUMBITS(0b10) [
            Off = 0b00,
            PullDownControl = 0b01,
            PullUpControl = 0b10
        ]
    ],
    // GPIO Pull-up/down Clock Register 0
    GPPUDCLK0 [
        // Assert Clock for pin 15
        PUDCLK15 OFFSET(15) NUMBITS(0b1) [
            NoEffect = 0b0,
            AssertClock = 0b1
        ],
        // Assert Clock for pin 14
        PUDCLK14 OFFSET(14) NUMBITS(0b1) [
            NoEffect = 0b0,
            AssertClock = 0b1
        ]
    ],
    // UART Flag Register
    FR [
        // Transmit FIFO Empty
        TXFE OFFSET(7) NUMBITS(0b1) [],
        // Transmit FIFO Full
        TXFF OFFSET(5) NUMBITS(0b1) [],
        // Recieve FIFO Empty
        RXFE OFFSET(4) NUMBITS(1) []
    ],
    // Integer Baud Rate Divisor
    IBRD [
        // 16 bit field
        IBRD OFFSET(0) NUMBITS(0b10000) []
    ],
    // Fractional Baud Rate Divisor
    FBRD [
        // 6 bit field
        FBRD OFFSET(0) NUMBITS(0b110) []
    ],
    // Line Control
    LCRH [
        // Word Length. We just use 8 bits like normal people.
        WLEN OFFSET(5) NUMBITS(0b10) [
            FiveBits = 0b00,
            SixBits = 0b01,
            SevenBits = 0b10,
            EightBits = 0b11
        ],
        // FIFO Enable
        FEN OFFSET(5) NUMBITS(0b1) [
            FifoDisabled = 0b0,
            FifoEnabled = 0b1
        ]
    ],

    // Control Register
    CR [
        // Receive Enable
        RXE OFFSET(9) NUMBITS(0b1) [],
        // Transmit Enable
        TXE OFFSET(8) NUMBITS(0b1) [],
        // UART ENable
        UARTEN OFFSET(0) NUMBITS(0b1) []
    ],
    // Interrupt Clear Register
    ICR [
        // 11 bit field
        ALL OFFSET(0) NUMBITS(0b1011) []
    ]
}
register_structs! {
    #[allow(non_snake_case)]
    pub gpio {
        (0x00 => _reserved0),
        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => _reserved1),
        (0x94 => GPPUD: ReadWrite<u32, GPPUD::Register>),
        (0x98 => GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>),
        (0x9c => @END),
    }
}
register_structs! {
    #[allow(non_snake_case)]
    pub uart {
        (0x00 => DR: ReadWrite<u32>),
        (0x04 => _reserved0),
        (0x18 => FR: ReadOnly<u32, FR::Register>),
        (0x1c => _reserved1),
        (0x24 => IBRD: WriteOnly<u32, IBRD::Register>),
        (0x28 => FBRD: WriteOnly<u32, FBRD::Register>),
        (0x2c => LCRH: WriteOnly<u32, LCRH::Register>),
        (0x30 => CR: WriteOnly<u32, CR::Register>),
        (0x34 => _reserved2),
        (0x44 => ICR: WriteOnly<u32, ICR::Register>),
        (0x48 => @END),
    }
}
pub struct PL011 {
    base: usize
}

impl PL011 {
    pub const fn new() -> Self {
        Self {
            base: 0x2000_0000
        }
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
        let gpio_regs = (self.base + 0x20_0000) as *const gpio;
        // map pins 14 and 15 to PL011 TX and RX respectively
        (*gpio_regs).GPFSEL1.modify(GPFSEL1::FSEL15::AltFunc0 + GPFSEL1::FSEL14::AltFunc0);
        // enable pins 14 and 15 by disabling pull up/down
        (*gpio_regs).GPPUD.write(GPPUD::PUD::Off);
        cpu::spin_for_cycles(150);
        // Assert Clock for both
        (*gpio_regs).GPPUDCLK0.write(GPPUDCLK0::PUDCLK15::AssertClock + GPPUDCLK0::PUDCLK14::AssertClock);
        cpu::spin_for_cycles(150);
        // Flush GPIO setup
        (*gpio_regs).GPPUDCLK0.set(0);

        let uart_regs = (self.base + 0x20_1000) as *const uart;
        // Turn off UART temporarily with CR (Control Register)
        (*uart_regs).CR.set(0);
        // clear all interrupts with ICR (Interrupt Clear Register)
        (*uart_regs).ICR.write(ICR::ALL::CLEAR);
        // set IBRD (Integer Baud Rate Divisor) to 13
        // because (48MHz/16)/230400 = 13.02083, margin of error is acceptable
        (*uart_regs).IBRD.write(IBRD::IBRD.val(13));
        // set FBRD (Fractional Baud Rate Divisor) to 1
        // because 0.02083*64 = 1.3312, rounded to 1
        (*uart_regs).FBRD.write(FBRD::FBRD.val(1));
        // set LCRH to 8 bit chars and enable FIFO
        (*uart_regs).LCRH.write(LCRH::WLEN::EightBits + LCRH::FEN::FifoEnabled);
        // set CR to enable UART, TX, and RX
        (*uart_regs).CR.write(CR::UARTEN::SET + CR::TXE::SET + CR::RXE::SET);
    }
    fn get(&mut self) -> Option<u8> {
        let uart_regs = (self.base + 0x20_1000) as *const uart;
        // match on emptiness of RX fifo
        unsafe {
            match (*uart_regs).FR.matches_all(FR::RXFE::SET) {
                true => None,
                false => Some((*uart_regs).DR.get() as u8)
            }
        }
    }
    fn put(&mut self, value: u8) {
        let uart_regs = (self.base + 0x20_1000) as *const uart;
        unsafe {
            while (*uart_regs).FR.matches_all(FR::TXFF::SET) {
                cpu::nop();
            }
            (*uart_regs).DR.set(value as u32);
        }
    }
}
use crate::driver_interfaces::{Console, Uart};
use register::{mmio::*, register_bitfields, register_structs};

// Info from the datasheet
register_bitfields! {
    u8,
    // Line Control Register
    LCR [
        // Word Length Select
        WLS OFFSET(0) NUMBITS(0b10) [
            FiveBits = 0b00,
            SixBits = 0b01,
            SevenBits = 0b10,
            EightBits = 0b11
        ],
        // Divisor Latch Access Bit
        DLAB OFFSET(7) NUMBITS(0b1) []
    ],
    // FIFO Control Register
    FCR [
        // Controls whether FIFOs are enabled or not
        FEN OFFSET(0) NUMBITS(0b1) []
    ],
    // Interrupt Enable Register
    IER [
        // Enable Received Data Available Interrupt
        ERBFI OFFSET(0) NUMBITS(0b1) [],
        // Enable Transmitter Holding Register Empty Interrupt
        ETBEI OFFSET(1) NUMBITS(0b1) [],
        // Enable Receiver Line Status Interrupt
        ELSI OFFSET(2) NUMBITS(0b1) [],
        // Enable MODEM Status Interrupt
        EDSSI OFFSET(3) NUMBITS(0b1) []
    ],
    // Line Status Register
    LSR [
        // Transmitter Empty
        TEMT OFFSET(6) NUMBITS(0b1) [],
        // Data Ready
        DR OFFSET(0) NUMBITS(0b1) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    pub UARTBlock {
        (0x00 => RBR: ReadWrite<u8>), // Can't easily make 2 registers for the same address unfortunately (both reading and writing takes place here, in addition to half of the divisor value)
        (0x01 => IER: ReadWrite<u8, IER::Register>),
        (0x02 => FCR: WriteOnly<u8, FCR::Register>),
        (0x03 => LCR: ReadWrite<u8, LCR::Register>),
        (0x04 => _reserved0),
        (0x05 => LSR: ReadWrite<u8, LSR::Register>),
        (0x06 => @END),
    }
}

#[derive(Debug, Clone)]
pub struct NS16550A {
    base_address: usize,
}

impl NS16550A {
    /// # Safety
    /// The given base address must be valid.
    pub const unsafe fn new(base_address: usize) -> Self {
        Self { base_address }
    }

    fn regs(&mut self) -> &UARTBlock {
        unsafe { &*(self.base_address as *const UARTBlock) }
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
    fn init(&mut self) {
        let regs = self.regs();
        // Word length = 8 bits
        regs.LCR.write(LCR::WLS::EightBits);
        // Enable FIFO
        regs.FCR.write(FCR::FEN::SET);
        // Enable receiver buffer interrupts
        regs.IER.write(IER::ERBFI::SET);
        // Set the divisor to 2400 baud or whatever
        // doesn't really do anything in qemu
        // open the DLB to set divisor
        // formula for finding divisor is in the datasheet
        // Assume 22.729 MHz clock
        regs.LCR.write(LCR::DLAB::SET + LCR::WLS::EightBits);
        // let divisor = 592u16;
        let divisor = 12u16;
        regs.RBR.set((divisor & 0xFF) as u8);
        regs.IER.set((divisor >> 8 & 0xff) as u8);
        // close the DLB for normal use
        regs.LCR.write(LCR::WLS::EightBits);
    }

    fn get(&mut self) -> Option<u8> {
        let regs = self.regs();
        // check if data is ready
        match regs.LSR.matches_all(LSR::DR::SET) {
            true => Some(regs.RBR.get()),
            false => None,
        }
    }

    fn put(&mut self, value: u8) {
        // safety: this function is only called if base address is valid.
        let regs = self.regs();
        // Wait for empty transmitter
        while regs.LSR.matches_all(LSR::TEMT::CLEAR) {}
        // just set the value
        regs.RBR.set(value)
    }
}

impl Console for NS16550A {
    fn init(&mut self) {
        Uart::init(self)
    }

    fn base_address(&self) -> usize {
        self.base_address
    }
}

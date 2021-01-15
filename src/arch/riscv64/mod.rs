use crate::util::UnsafeMutex;

global_asm!(include_str!("header.S"));
global_asm!(include_str!("trap.S"));

pub mod cpu;
pub mod drivers;
pub mod mmu;
pub mod time;
pub mod trap;

pub static INTERRUPT_CONTROLLER: UnsafeMutex<drivers::CLINT> =
    UnsafeMutex::new(drivers::CLINT::uninit());

pub type Regs = [usize; 32];
pub type Fregs = [f64; 32];

pub const fn default_regs() -> Regs {
    [0; 32]
}
pub const fn default_fregs() -> Fregs {
    [0.; 32]
}

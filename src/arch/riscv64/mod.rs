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

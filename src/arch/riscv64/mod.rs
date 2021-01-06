use crate::util::UnsafeMutex;

global_asm!(include_str!("header.S"));

pub mod cpu;
pub mod drivers;
pub mod mmu;
pub mod time;

pub static INTERRUPT_CONTROLLER: UnsafeMutex<drivers::CLINT> =
    UnsafeMutex::new(drivers::CLINT::uninit());

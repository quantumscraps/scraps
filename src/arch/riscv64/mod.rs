use spin::Mutex;

global_asm!(include_str!("header.S"));

pub mod cpu;
pub mod drivers;
pub mod mmu;
pub mod time;

pub static INTERRUPT_CONTROLLER: Mutex<drivers::CLINT> = Mutex::new(drivers::CLINT::new(0));

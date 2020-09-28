#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/time.rs"]
mod arch_time;

#[cfg(target_arch = "riscv64")]
#[path = "arch/riscv/time.rs"]
mod arch_time;

pub use arch_time::*;
use core::time::Duration;

pub trait TimeCounter {
    fn accuracy(&self) -> Duration;
    fn uptime(&self) -> Duration;
    fn wait_for(&self, duration: Duration);   
}
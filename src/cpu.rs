#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/cpu.rs"]
mod arch_cpu;

#[cfg(target_arch = "riscv")]
#[path = "arch/riscv/cpu.rs"]
mod arch_cpu;

pub use arch_cpu::*;

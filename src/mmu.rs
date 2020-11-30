#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mmu.rs"]
mod arch_mmu;

#[cfg(target_arch = "riscv64")]
#[path = "arch/riscv/mmu.rs"]
mod arch_mmu;

pub use arch_mmu::*;

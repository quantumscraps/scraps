#[cfg(target_arch = "riscv64")]
#[path = "arch/riscv/lock.rs"]
mod lock;

#[cfg(target_arch = "riscv64")]
pub use lock::*;

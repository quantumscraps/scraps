
#[cfg(feature = "bsp_raspi64")]
mod raspi64;

#[cfg(feature = "bsp_raspi64")]
pub use raspi64::*;

#[cfg(feature = "bsp_riscvirt")]
mod riscvirt;

#[cfg(feature = "bsp_riscvirt")]
pub use riscvirt::*;

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "riscv64")]
mod riscv64;

macro_rules! pub_mod {
    ($mod:ident) => {
        pub mod $mod {
            #[cfg(target_arch = "aarch64")]
            pub use $crate::arch::aarch64::$mod::*;
            #[cfg(target_arch = "riscv64")]
            pub use $crate::arch::riscv64::$mod::*;
        }
    };
    ($($mod:ident),+) => {
        $(pub_mod!($mod);)+
    };
}

pub_mod!(cpu, time, mmu);

global_asm!(include_str!("header.S"));
use crate::{link_var, memory};
use cortex_a::{asm, regs::*};

link_var!(__start);
#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe();
    }
}

pub use asm::nop;

#[inline(always)]
pub fn spin_for_cycles(n: usize) {
    for _ in 0..n {
        nop();
    }
}
#[inline(always)]
pub fn core_num() -> u8 {
    // technically there can be 255 cores per clusters and 255 clusters,
    // but for now we're just assuming there's one cluster
    (MPIDR_EL1.get() & 0xFF) as u8
}
#[inline(always)]
pub fn cluster_num() -> u8 {
    ((MPIDR_EL1.get() >> 8) & 0xFF) as u8
}
#[inline(always)]
#[no_mangle]
/// # Safety
/// Safe only to call from asm entry. Same safety restrictions as
/// [setup_environment].
///
/// [setup_environment]: memory::setup_environment
pub unsafe fn __early_entry(_dtb_addr: *mut u8) -> ! {
    if cluster_num() != 0 {
        wait_forever()
    }
    if core_num() != 0 {
        wait_forever()
    }
    match CurrentEL.get() & 0b11_00 {
        0b11_00 => el3_to_el2(),
        0b10_00 => el2_to_el1(),
        0b01_00 => memory::setup_environment(_dtb_addr),
        _ => wait_forever(),
    }
}

#[inline(always)]
/// # Safety
/// Only safe to call from [__early_entry].
unsafe fn el2_to_el1() -> ! {
    // grant Counting and Timer for EL1
    CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCTEN::SET + CNTHCTL_EL2::EL1PCEN::SET);

    // Counter-timer Virtual Offset = 0
    CNTVOFF_EL2.set(0);

    // Enable aarch64 (not aarch32, since that's an option!)
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);

    // Set the Debug exception mask, SError interrupt mask, IRQ interrupt mask, FIQ interrupt mask, and EL1h for selected stack pointer
    SPSR_EL2.write(
        SPSR_EL2::D::Masked
            + SPSR_EL2::A::Masked
            + SPSR_EL2::I::Masked
            + SPSR_EL2::F::Masked
            + SPSR_EL2::M::EL1h,
    );

    // Set the link register to the correct location (this will execute after exception return)
    ELR_EL2.set(memory::setup_environment as *const () as u64);

    // Set up the Stack Pointer
    SP_EL1.set(&__start as *const _ as u64);

    // Perform exception return
    asm::eret()
}

#[inline(always)]
/// # Safety
/// Only safe to call from [__early_entry].
unsafe fn el3_to_el2() -> ! {
    // This is RW bit + Hypervisor Call Enable + Non-secure bit. I'll open a PR in cortex-a to add HCE
    SCR_EL3.set(0x5b1);

    // Set the Debug exception mask, SError interrupt mask, IRQ interrupt mask, FIQ interrupt mask, and EL2h for selected stack pointer
    //SPSR_EL3.write(SPSR_EL3::D::Masked + SPSR_EL3::A::Masked + SPSR_EL3::I::Masked + SPSR_EL3::F::Masked + SPSR_EL3::M::EL2h);
    //TODO: why does this say use of undeclared type??? the following does essentially the same thing
    SPSR_EL3.set(0x3c9);
    // Set up the link register to run the el2 to el1 drop
    ELR_EL3.set(el2_to_el1 as *const () as u64);

    // Perform exception return
    asm::eret()
}

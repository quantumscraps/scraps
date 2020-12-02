use cortex_a::{barrier, regs::*};

#[naked]
#[inline(never)]
pub unsafe fn init() {
    /*
    // Initialize the Translation Control Register
    TCR_EL1.write(TCR_EL1::TG1::KiB_4 // 4 KiB granule size 
        + TCR_EL1::IPS::Bits_48 // 48 bit addresses (256TiB address space)

    )
    */
}
global_asm!(include_str!("header.S"));
use cortex_a::asm;
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
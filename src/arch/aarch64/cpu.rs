global_asm!(include_str!("header.S"));

#[inline(always)]
pub fn wait_forever() -> ! {
    unsafe {
        loop {
            #[rustfmt::skip]
            asm!(
                "wfe",
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}

#[inline(always)]
pub fn nop() {
    unsafe { asm!("nop") }
}

#[inline(always)]
pub fn spin_for_cycles(n: usize) {
    for _ in 0..n {
        nop();
    }
}

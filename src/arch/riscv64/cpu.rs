global_asm!(include_str!("header.S"));

#[inline(always)]
pub fn wait_forever() -> ! {
    // Safety: Never returns
    unsafe {
        loop {
            #[rustfmt::skip]
            asm!(
                "wfi",
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}

// pub fn nop() {
//     unsafe { asm!("nop") }
// }

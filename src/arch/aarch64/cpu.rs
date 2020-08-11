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

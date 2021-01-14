use crate::util::HeaplessResult;

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

/// # Safety
/// Only safe to call from asm entry.
#[no_mangle]
pub unsafe extern "C" fn __early_entry(_: *const i8, dtb_addr: *mut u8) -> ! {
    // Check hart
    let hart_id: u64;
    asm!("csrr {0}, mhartid", out(reg) hart_id);
    if hart_id != 0 {
        wait_forever()
    }

    // Enable interrupts and supervisor mode

    //                 ~~~~~~~~~~ MPP = 1 (S-mode)
    //                              ~~~~~~ SPIE = 1 (enable S-mode interrupts)
    //                                       ~~~~~~ SIE = 1
    let mstatus: u64 = 0b01 << 11 | 1 << 5 | 1 << 1;
    asm!("csrw mstatus, {0}", in(reg) mstatus);

    asm!("csrw mepc, {0}", in(reg) crate::memory::setup_environment);

    extern "C" {
        fn asm_trap_vector();
    }
    asm!("csrw stvec, {0}", in(reg) asm_trap_vector);
    asm!("csrw sscratch, {0}", in(reg) &mut super::trap::__trap_frame);

    asm!("csrw mie, zero");
    // all exceptions and interrupts go to S-mode
    asm!("csrw medeleg, {0}", in(reg) u64::MAX);
    asm!("csrw mideleg, {0}", in(reg) u64::MAX);

    //             ~~~~~~ STIE = 1 (timer interrupt)
    //                      ~~~~~~ SSIE = 1 (software interrupt)
    let sie: u64 = 1 << 5 | 1 << 1;
    asm!("csrw sie, {0}", in(reg) sie);
    asm!("csrw satp, zero");

    asm!("mv ra, {0}", in(reg) __early_entry_return, lateout("ra") _);

    // reset dtb in proper register
    asm!("mv a1, {0}", in(reg) dtb_addr, lateout("a1") _);

    // return
    asm!("mret", options(noreturn));
}

/// # Safety
/// Don't call.
#[allow(improper_ctypes_definitions)]
unsafe extern "C" fn __early_entry_return(r: HeaplessResult<!>) -> ! {
    crate::panic::print_backtrace_from_kinit(r);
    wait_forever()
}

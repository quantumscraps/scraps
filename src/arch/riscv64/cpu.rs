use crate::{
    link_var,
    mmu::{PageTable, Permissions, HIGHER_HALF_BASE},
    util::HeaplessResult,
};

use super::mmu::{SvTable, __root_page_table, ONEGIG};

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

    asm!("csrw mie, zero");
    // all exceptions and interrupts go to S-mode
    asm!("csrw medeleg, {0}", in(reg) u64::MAX);
    asm!("csrw mideleg, {0}", in(reg) u64::MAX);

    //             ~~~~~~ STIE = 1 (timer interrupt)
    //                      ~~~~~~ SSIE = 1 (software interrupt)
    let sie: u64 = 1 << 5 | 1 << 1;
    asm!("csrw sie, {0}", in(reg) sie);

    // Setup root page table and set SATP
    link_var!(__kern_start, __kern_end);
    let kern_start = &__kern_start as *const _ as u64;
    let kern_end = &__kern_end as *const _ as u64;
    let begin_index = kern_start / ONEGIG;
    let end_index = kern_end / ONEGIG;
    for i in 0..=(end_index - begin_index) {
        let phys_addr = (i + begin_index) * ONEGIG;
        let hh_addr = (i * ONEGIG) + (HIGHER_HALF_BASE as u64);
        __root_page_table.map_gigapage(phys_addr, phys_addr, Permissions::RWX.into());
        __root_page_table.map_gigapage(hh_addr, phys_addr, Permissions::RWX.into());
    }

    // asm!("csrw satp, {0}", in(reg) &__root_page_table);
    __root_page_table.enable();

    asm!("csrw mepc, {0}", in(reg) (crate::memory::setup_environment as usize) - (kern_start as usize) + HIGHER_HALF_BASE);
    extern "C" {
        fn asm_trap_vector();
    }
    asm!("csrw stvec, {0}", in(reg) (asm_trap_vector as usize) - (kern_start as usize) + HIGHER_HALF_BASE);
    asm!("csrw sscratch, {0}", in(reg) (&mut super::trap::__trap_frame as *mut _ as usize) - (kern_start as usize) + HIGHER_HALF_BASE);

    // return
    asm!(
        "mret",
        in("ra") (wait_forever as usize) - (kern_start as usize) + HIGHER_HALF_BASE,
        in("a0") dtb_addr,
        in("a1") kern_start,
        options(noreturn),
    );
}

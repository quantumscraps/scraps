use crate::{
    drivers::known_good_uart,
    link_var,
    mmu::{PageTable, Permissions, HIGHER_HALF_BASE},
};

use super::{
    drivers,
    mmu::{SvTable, __root_page_table, ONEGIG},
    Regs,
};

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
pub extern "C" fn __early_entry(_: *const i8, dtb_addr: *mut u8) -> ! {
    // Setting up all m* csrs before returning to S-mode early_entry2

    // Check hart
    let hart_id: u64;
    unsafe { asm!("csrr {0}, mhartid", out(reg) hart_id) };
    if hart_id != 0 {
        wait_forever()
    }

    // setup uart for super early printk
    unsafe { *crate::STDOUT.get_mut() = Some(known_good_uart()) };

    // Enable interrupts and supervisor mode

    //                 ~~~~~~~~~~ MPP = 1 (S-mode)
    //                              ~~~~~~ SPIE = 1 (enable S-mode interrupts)
    //                                       ~~~~~~ SIE = 1
    let mstatus: u64 = 0b01 << 11 | 1 << 5 | 1 << 1;
    unsafe {
        asm!("csrw mstatus, {0}", in(reg) mstatus);

        asm!("csrw mie, zero");
        // all exceptions and interrupts go to S-mode
        asm!("csrw medeleg, {0}", in(reg) u64::MAX);
        asm!("csrw mideleg, {0}", in(reg) u64::MAX);

        // Setup root page table and set SATP
        // link_var!(__kern_start, __kern_end);
        // let kern_start = &__kern_start as *const _ as usize;
        // let kern_end = &__kern_end as *const _ as usize;
        // let begin_index = kern_start / ONEGIG;
        // let end_index = kern_end / ONEGIG;
        // for i in 0..=(end_index - begin_index) {
        //     let phys_addr = (i + begin_index) * ONEGIG;
        //     let hh_addr = (i * ONEGIG) + HIGHER_HALF_BASE;
        //     __root_page_table.map_gigapage(phys_addr, phys_addr, Permissions::RWX.into());
        //     __root_page_table.map_gigapage(hh_addr, phys_addr, Permissions::RWX.into());
        // }

        // // asm!("csrw satp, {0}", in(reg) &__root_page_table);
        // __root_page_table.enable();

        asm!("csrw mepc, {0}", in(reg) early_entry2);
        asm!("mret", in("a0") dtb_addr, options(noreturn));
    }
}

extern "C" fn early_entry2(dtb_addr: *mut u8) -> ! {
    // Setup paging and return to higher half early_entry3

    //             ~~~~~~ STIE = 1 (timer interrupt)
    //                      ~~~~~~ SSIE = 1 (software interrupt)
    let sie: u64 = 1 << 5 | 1 << 1;
    unsafe { asm!("csrw sie, {0}", in(reg) sie) };

    link_var!(__kern_start);
    let kern_start = unsafe { &__kern_start } as *const _ as usize;

    // setup sscratch and stvec which will be touched by mmu init
    extern "C" {
        fn asm_trap_vector();
    }
    unsafe {
        asm!("csrw stvec, {0}", in(reg) asm_trap_vector);
        asm!("csrw sscratch, {0}", in(reg) &mut super::trap::__trap_frame);
        crate::printk!(
            "sscratch = {:x}",
            &mut super::trap::__trap_frame as *mut _ as usize
        );
    }

    // setup paging and return to kinit
    unsafe {
        super::mmu::init(
            crate::kinit as usize,
            (wait_forever as usize) - kern_start + HIGHER_HALF_BASE,
            dtb_addr as usize,
            0,
        );
    }

    // unsafe {
    //     asm!("csrw sepc, {0}", in(reg) (crate::memory::setup_environment as usize) - kern_start + HIGHER_HALF_BASE);

    //     // return
    //     asm!(
    //         "mret",
    //         in("ra") (wait_forever as usize) - kern_start + HIGHER_HALF_BASE,
    //         in("a0") dtb_addr,
    //         in("a1") kern_start,
    //         options(noreturn),
    //     );
    // }
}

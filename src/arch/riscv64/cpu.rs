use crate::{bsp::UART, printk2};

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

#[repr(C)]
#[derive(Clone)]
pub struct TrapFrame {
    pub regs: [usize; 32],
    pub fregs: [f64; 32],
    pub satp: usize,
    pub trap_stack: *mut u8,
    pub hartid: usize,
}

impl TrapFrame {
    const fn from_stack(sp: *mut u8) -> Self {
        Self {
            regs: [0; 32],
            fregs: [0.; 32],
            satp: 0,
            trap_stack: sp,
            hartid: 0,
        }
    }
}

/// This is written to mscratch to store the trap frame.
#[allow(non_upper_case_globals)]
#[no_mangle]
static mut __trap_frame: TrapFrame = TrapFrame::from_stack(unsafe {
    (__trap_stack.as_mut_ptr()).add(core::mem::size_of_val(&__trap_stack))
});

/// Stack storage. 1kb to encourage keeping trap handlers small.
#[allow(non_upper_case_globals)]
static mut __trap_stack: [u8; 1024] = [0; 1024];

#[allow(non_upper_case_globals)]
pub const mtime: *const u64 = 0x0200_bff8 as *const u64;
#[allow(non_upper_case_globals)]
pub const mtimecmp: *mut u64 = 0x0200_4000 as *mut u64;

#[no_mangle]
extern "C" fn trap_vector(
    epc: usize,
    tval: usize,
    cause: usize,
    hart: usize,
    status: usize,
    frame: &mut TrapFrame,
) -> usize {
    let is_async = cause >> 63 & 1 == 1;
    let cause_num = cause & 0xfff;
    let mut return_pc = epc;
    let uart_mut = unsafe { crate::util::get_mutex_mut(&UART) };
    // let orig_uart_locked = UART.is_locked();
    // unsafe { UART.force_unlock() };

    printk2!(
        uart_mut,
        "Interrupt epc={} tval={} cause={} hart={} status={} frame={}",
        epc,
        tval,
        cause,
        hart,
        status,
        frame as *mut _ as usize
    );
    // panic_println!("Stuff happened");

    if is_async {
        match cause_num {
            // timer
            7 => {
                printk2!(uart_mut, "Timer interrupt! mtime = {}", unsafe { *mtime });
                printk2!(uart_mut, "Rescheduling mtimecmp to 2s from now...");
                unsafe { mtimecmp.write_volatile(mtime.read_volatile() + 20_000_000) };
            }
            _ => {}
        }
    } else {
        match cause_num {
            // page fault
            15 => {
                printk2!(uart_mut, "Page fault... skipping to next instruction");
                return_pc += 4;
            }
            _ => {}
        }
    }

    // Set locked state back to normal
    // if orig_uart_locked {
    //     core::mem::forget(UART.lock());
    // }

    printk2!(uart_mut, "Returning from interrupt");

    return_pc
}

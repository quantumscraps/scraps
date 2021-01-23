use crate::{
    arch::mmu::{SvTable, __root_page_table},
    link_var,
    mmu::HIGHER_HALF_BASE,
    util::HeaplessResult,
};
link_var!(__bss_start);
link_var!(__bss_end);

const fn subdivide_size<T: Sized>(size: usize) -> usize {
    let t_size = core::mem::size_of::<T>();
    assert!(
        size % t_size == 0,
        "bss size must be a multiple of given type"
    );
    size / t_size
}

/// # Safety
/// Safe only to be called from asm entry.
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn setup_environment(dtb_addr: *mut u8, old_kern_start: u64) {
    // setup stack and gp too
    let gp: u64;
    let sp: u64;
    // let ra: u64;
    asm!("mv {0}, gp", out(reg) gp);
    asm!("mv {0}, sp", out(reg) sp);
    // asm!("mv {0}, ra", out(reg) ra);
    asm!("mv gp, {0}", in(reg) gp - old_kern_start + (HIGHER_HALF_BASE as u64));
    asm!("mv sp, {0}", in(reg) sp - old_kern_start + (HIGHER_HALF_BASE as u64));

    // Unmap old page table
    // __root_page_table.unmap_gigapage(old_kern_start);

    // get bss section as slice
    let slice = core::slice::from_raw_parts_mut(
        &__bss_start as *const _ as *mut usize,
        subdivide_size::<usize>(
            &__bss_end as *const _ as usize - &__bss_start as *const _ as usize,
        ),
    );

    // zero the slice
    for thing in slice.iter_mut() {
        *thing = 0;
    }

    // run kinit
    core::mem::transmute::<_, extern "C" fn(*mut u8, u64)>(
        (crate::kinit as usize) - (old_kern_start as usize) + HIGHER_HALF_BASE,
    )(dtb_addr, old_kern_start)
}

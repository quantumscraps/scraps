use crate::{link_var, util::HeaplessResult};
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
pub unsafe extern "C" fn setup_environment(dtb_addr: *mut u8) -> HeaplessResult<!> {
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
    crate::kinit(dtb_addr)
}
